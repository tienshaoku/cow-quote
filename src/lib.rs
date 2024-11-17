mod constant;
mod format;
mod ierc20;
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
    cow_post_quote_api::cowswap_post_quote,
    zerox_get_quote_api::get_zerox_price_quote,
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
    let provider = Provider::<Ws>::connect(secret::ETH_RPC).await?;
    let provider = Arc::new(provider);

    let settlement_contract = constant::GPv2SETTLEMENT.parse::<Address>()?;
    let trade_filter = Filter::new().address(settlement_contract);

    let trade_event = TradeEvent::new::<_, Provider<Ws>>(trade_filter, Arc::clone(&provider));
    let mut stream = trade_event.stream().await?.with_meta().take(TRIAL_COUNT);
    while let Some(Ok((trade, meta))) = stream.next().await {
        // println!("Trade: {:#?}\n", trade);
        let order_uid = trade.order_uid;

        let cow_api_response: CowGetResponse = cowswap_get_order(&order_uid.to_string()).await?;
        // println!("Order Response: {:#?}\n", cow_api_response);

        // 0x has only sell orders
        if cow_api_response.is_sell() && cow_api_response.sell() == cow_api_response.executed_sell()
        {
            let timestamp = if let Some(block) = &provider.get_block(meta.block_number).await? {
                block.timestamp
            } else {
                U256::zero()
            };

            let mut order = Order::from_cow_api_response(
                Arc::clone(&provider),
                order_uid.to_string(),
                meta.block_number.as_u64(),
                timestamp.as_u64(),
                &cow_api_response,
            )
            .await?;

            let zerox_response = get_zerox_price_quote(
                "1",
                cow_api_response.sell_token(),
                cow_api_response.buy_token(),
                cow_api_response.sell(),
                cow_api_response.owner(),
            )
            .await?;
            // println!("0x Response: {:#?}\n", zerox_response);

            order.update_zerox_comparison(zerox_response);

            let cows_own_quote_buy = cowswap_post_quote(
                cow_api_response.owner(),
                cow_api_response.sell_token(),
                cow_api_response.buy_token(),
                cow_api_response.sell(),
            )
            .await?;

            order.update_cows_own_quote_comparison(&cows_own_quote_buy);
            println!("Order: {:#?}\n", order);
        }
    }

    Ok(())
}
