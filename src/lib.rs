mod constant;
mod format;
mod ierc20;
mod order;
#[path = "secret.rs"]
mod secret;
pub mod services;

use ethers::{
    contract::EthEvent,
    providers::{Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use futures::StreamExt;
use order::Order;
use serde::{Deserialize, Serialize};
use services::cow_api::{get_cowswap_order, CowAPIResponse};
use services::zerox_api::get_zerox_price_quote;
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

pub async fn run() -> eyre::Result<()> {
    let provider = Provider::<Ws>::connect(secret::ETH_RPC).await?;
    let provider = Arc::new(provider);

    let settlement_contract = constant::GPv2SETTLEMENT.parse::<Address>()?;
    let trade_filter = Filter::new().address(settlement_contract);

    let trade_event = TradeEvent::new::<_, Provider<Ws>>(trade_filter, Arc::clone(&provider));
    let mut stream = trade_event.stream().await?.with_meta().take(1);
    while let Some(Ok((trade, meta))) = stream.next().await {
        println!("Trade: {:#?}\n", trade);
        let order_uid = trade.order_uid;

        let order_response: CowAPIResponse = get_cowswap_order(&order_uid.to_string()).await?;
        println!("Order Response: {:#?}\n", order_response);

        let order = Order::from_cow_api_response(
            Arc::clone(&provider),
            order_uid.to_string(),
            &order_response,
        )
        .await?;

        println!("Order: {:#?}\n", order);

        let quote = get_zerox_price_quote(
            "1",
            order_response.sell_token(),
            order_response.buy_token(),
            order_response.sell(),
            order_response.owner(),
        )
        .await?;
        println!("{:#?}", quote);
    }

    Ok(())
}
