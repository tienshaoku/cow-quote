mod constant;
mod format;
mod ierc20_abi;
mod order;
#[path = "secret.rs"]
mod secret;
pub mod services;

use ethers::{
    contract::EthEvent,
    providers::{Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use format::format_decimals;
use futures::StreamExt;
use ierc20_abi::IERC20;
use order::Order;
use serde::{Deserialize, Serialize};
use services::cow_api::*;
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
    let mut stream = trade_event.stream().await?.with_meta().take(3);
    while let Some(Ok((trade, meta))) = stream.next().await {
        println!("Trade: {:#?}\n", trade);
        let order_uid = trade.order_uid;

        let order_response: OrderResponse = get_cowswap_order(&order_uid.to_string()).await?;
        println!("Order Response: {:#?}\n", order_response);

        let is_sell_weth = order_response.sell_token() == constant::WETH;
        let is_buy_weth = order_response.buy_token() == constant::WETH;

        let mut sell_token_decimals = 0;
        let mut buy_token_decimals = 0;

        let sell_token = order_response.sell_token().parse::<Address>()?;
        if is_sell_weth {
            sell_token_decimals = 18;
        } else {
            let erc20 = IERC20::new(sell_token, Arc::clone(&provider));
            sell_token_decimals = erc20.decimals().call().await? as u32;
        }

        let buy_token = order_response.buy_token().parse::<Address>()?;
        if is_buy_weth {
            buy_token_decimals = 18;
        } else {
            let erc20 = IERC20::new(buy_token, Arc::clone(&provider));
            buy_token_decimals = erc20.decimals().call().await? as u32;
        }

        let executed_buy = format_decimals(&order_response.executed_buy(), buy_token_decimals);
        let executed_sell = format_decimals(&order_response.executed_sell(), sell_token_decimals);
        let buy = format_decimals(&order_response.buy(), buy_token_decimals);
        let sell = format_decimals(&order_response.sell(), sell_token_decimals);

        let mut order = Order::from(order_uid.to_string(), order_response.is_sell());
        order.update_surplus(
            &executed_buy,
            &executed_sell,
            &buy,
            &sell,
            order_response.is_sell(),
        );

        println!("Order: {:#?}\n", order);
    }

    Ok(())
}
