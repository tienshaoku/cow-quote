mod constant;
mod contract;
mod format;
mod order;
#[path = "secret.rs"]
mod secret;
pub mod services;

use ethers::{
    contract::EthEvent,
    middleware::Middleware,
    providers::{Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use futures::StreamExt;
use order::Order;
use serde::{Deserialize, Serialize};
use services::{
    cow_get_order_api::{cowswap_get_order, CowGetResponse},
    cow_post_quote_api::cowswap_quote_buy,
    uni_fork_swap::uni_swap_buy,
    zerox_get_quote_api::zerox_get_quote,
};
use std::sync::Arc;

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

const TRIAL_COUNT: usize = 4;

pub async fn run() -> eyre::Result<()> {
    let wss_provider = Provider::<Ws>::connect(secret::WSS_ETH_RPC).await?;
    let wss_provider = Arc::new(wss_provider);

    let settlement_contract = constant::GPv2SETTLEMENT.parse::<Address>()?;
    let trade_filter = Filter::new().address(settlement_contract);

    let trade_event = TradeEvent::new::<_, Provider<Ws>>(trade_filter, Arc::clone(&wss_provider));
    let mut stream = trade_event.stream().await?.with_meta().take(TRIAL_COUNT);
    while let Some(Ok((trade, meta))) = stream.next().await {
        let order_uid = trade.order_uid;

        let cow_api_response: CowGetResponse = cowswap_get_order(&order_uid.to_string()).await?;

        // TODO: see if can throw this into a thread
        // 0x has only sell orders
        if cow_api_response.is_sell() && cow_api_response.sell() == cow_api_response.executed_sell()
        {
            let block_number = meta.block_number.as_u64();
            println!("New settlement found at block number: {:?}", block_number);

            let timestamp = match wss_provider.get_block(block_number).await? {
                Some(block) => block.timestamp,
                None => continue,
            };

            let owner = cow_api_response.owner();
            let sell_token = cow_api_response.sell_token();
            let sell_amount = cow_api_response.sell();
            let buy_token = cow_api_response.buy_token();

            let mut order = Order::from_cow_api_response(
                Arc::clone(&wss_provider),
                order_uid.to_string(),
                block_number,
                timestamp.as_u64(),
                &cow_api_response,
            )
            .await?;

            let zerox_response =
                zerox_get_quote("1", sell_token, buy_token, sell_amount, owner).await?;

            // TODO: include gas cost; complicated calculation
            order.update_zerox_comparison(zerox_response);

            let cows_own_quote_buy =
                cowswap_quote_buy(owner, sell_token, buy_token, sell_amount).await?;
            order.update_cows_own_quote_comparison(&cows_own_quote_buy);

            let uni_swapped_buy =
                uni_swap_buy(block_number, owner, sell_token, buy_token, sell_amount).await?;
            order.update_univ3_swap_comparison(&uni_swapped_buy);

            println!("Order: {:#?}\n", order);
        }
    }

    Ok(())
}
