mod constant;
mod contract;
pub mod helper;
pub mod order;
pub mod services;

use ethers::{
    contract::EthEvent,
    middleware::Middleware,
    providers::{Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use futures::StreamExt;
use helper::EnvConfig;
use order::Order;
use serde::{Deserialize, Serialize};
use services::{
    aws_dynamodb::DynamoDbClient, cow_get_order_api::cowswap_get_order,
    cow_post_quote_api::cowswap_quote_buy, uni_fork_swap::uni_swap_buy,
    zerox_get_quote_api::zerox_quote_buy,
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
#[ethevent(name = "Trade")]
pub struct TradeEvent {
    #[ethevent(indexed)]
    pub owner: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: U256,
    pub buy_amount: U256,
    pub fee_amount: U256,
    pub order_uid: Bytes,
}

macro_rules! fetch_quote_and_update_order {
    ($quote_fn:expr, $order:expr, $update_method:ident, $error_msg:expr) => {
        match $quote_fn.await {
            Ok(quote) => $order.$update_method(&quote),
            Err(e) => {
                eprintln!("{}: {}", $error_msg, e);
            }
        }
    };
}

pub async fn handle_start_service() -> eyre::Result<String> {
    tokio::spawn(run_with_timeout());

    Ok("Blockchain service has started".to_string())
}

pub async fn run_with_timeout() -> eyre::Result<String> {
    // TODO: change back to 15 mins before live
    let duration = 2 * 60;

    tokio::select! {
        _ = run() => Err(eyre::eyre!("Service error")),
        // use timeout to end the service
        _ = tokio::time::sleep(Duration::from_secs(duration)) => {
            let message = format!("Service has ended after {} secs", duration);
            println!("{}", message);
            Ok(message)
        }
    }
}

pub async fn run() -> eyre::Result<()> {
    let config = EnvConfig::new();

    let wss_provider = Provider::<Ws>::connect(config.get_alchemy_wss_url()).await?;
    let wss_provider = Arc::new(wss_provider);

    let aws_client = DynamoDbClient::new().await?;

    let settlement_contract = constant::GPv2SETTLEMENT.parse::<Address>()?;
    let trade_filter = Filter::new().address(settlement_contract);

    let trade_event = TradeEvent::new::<_, Provider<Ws>>(trade_filter, Arc::clone(&wss_provider));
    let mut stream = trade_event.stream().await?.with_meta();
    while let Some(Ok((trade, meta))) = stream.next().await {
        let order_uid = trade.order_uid;

        let api_client = reqwest::Client::new();
        let (cow_api_response, should_proceed) =
            cowswap_get_order(&api_client, &order_uid.to_string()).await?;

        // TODO: see if can throw this into a thread
        if should_proceed {
            let block_number = meta.block_number.as_u64();
            println!("New settlement found at block number: {:?}", block_number);

            let timestamp = match wss_provider.get_block(block_number).await? {
                Some(block) => block.timestamp.as_u64(),
                None => 0,
            };

            let mut order = Order::from_cow_api_response(
                Arc::clone(&wss_provider),
                order_uid.to_string(),
                block_number,
                timestamp,
                &cow_api_response,
            )
            .await?;

            let owner = cow_api_response.owner();
            let sell_token = cow_api_response.sell_token();
            let sell_amount = cow_api_response.sell();
            let buy_token = cow_api_response.buy_token();

            fetch_quote_and_update_order!(
                zerox_quote_buy(
                    &config,
                    &api_client,
                    "1",
                    owner,
                    sell_token,
                    buy_token,
                    sell_amount
                ),
                order,
                update_zerox_comparison,
                "0x get quote failed"
            );

            fetch_quote_and_update_order!(
                cowswap_quote_buy(&api_client, owner, sell_token, buy_token, sell_amount),
                order,
                update_cows_own_quote_comparison,
                "CowSwap own quote failed"
            );

            fetch_quote_and_update_order!(
                uni_swap_buy(
                    &config,
                    block_number,
                    owner,
                    sell_token,
                    buy_token,
                    sell_amount
                ),
                order,
                update_univ3_swap_comparison,
                "Uni fork swap failed"
            );

            if order.no_successful_quote_at_all() {
                eprintln!("No successful quote at all");
                continue;
            }

            if let Err(e) = aws_client.upload_order(&order).await {
                eprintln!("Failed to upload order {}: {}", order.uid(), e);
                continue;
            }

            println!("Order: {:#?}\n", order);
        }
    }

    Ok(())
}
