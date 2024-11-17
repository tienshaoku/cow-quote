mod constant;
mod format;
mod ierc20_abi;
#[path = "secret.rs"]
mod secret;

use ethers::{
    contract::EthEvent,
    providers::{Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use format::{format_decimals, format_into_four_decimal_point};
use futures::StreamExt;
use ierc20_abi::IERC20;
use reqwest;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
struct OrderResponse {
    owner: String,
    #[serde(rename = "buyToken")]
    buy_token: String,
    #[serde(rename = "sellToken")]
    sell_token: String,
    #[serde(rename = "buyAmount")]
    buy: String,
    #[serde(rename = "sellAmount")]
    sell: String,
    #[serde(rename = "executedBuyAmount")]
    executed_buy: String,
    #[serde(rename = "executedSellAmount")]
    executed_sell: String,
    kind: String,
}

#[derive(Debug)]
struct Order {
    uid: String,
    is_sell: bool,
    buy: String,
    sell: String,
    executed_buy: String,
    executed_sell: String,
    net_surplus: String,
    surplus_percentage: String,
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

        let order_response = get_order(&order_uid.to_string()).await?;
        println!("Order Response: {:#?}\n", order_response);

        let is_sell_weth = order_response.sell_token == constant::WETH;
        let is_buy_weth = order_response.buy_token == constant::WETH;

        let mut sell_token_decimals = 0;
        let mut buy_token_decimals = 0;

        let sell_token = order_response.sell_token.parse::<Address>()?;
        if is_sell_weth {
            sell_token_decimals = 18;
        } else {
            let erc20 = IERC20::new(sell_token, Arc::clone(&provider));
            sell_token_decimals = erc20.decimals().call().await? as u32;
        }

        let buy_token = order_response.buy_token.parse::<Address>()?;
        if is_buy_weth {
            buy_token_decimals = 18;
        } else {
            let erc20 = IERC20::new(buy_token, Arc::clone(&provider));
            buy_token_decimals = erc20.decimals().call().await? as u32;
        }

        let order = compute_surplus(
            order_uid.to_string(),
            &order_response,
            sell_token_decimals,
            buy_token_decimals,
        );
        println!("Order: {:#?}\n", order);
    }

    Ok(())
}

async fn get_order(order_uid: &str) -> Result<OrderResponse, reqwest::Error> {
    let url: String = format!("https://api.cow.fi/mainnet/api/v1/orders/{}", order_uid);

    let response = reqwest::get(&url).await?.json::<OrderResponse>().await?;

    Ok(response)
}

fn compute_surplus(
    uid: String,
    response: &OrderResponse,
    sell_token_decimals: u32,
    buy_token_decimals: u32,
) -> Order {
    let executed_buy = format_decimals(&response.executed_buy, buy_token_decimals);
    let executed_sell = format_decimals(&response.executed_sell, sell_token_decimals);
    let buy = format_decimals(&response.buy, buy_token_decimals);
    let sell = format_decimals(&response.sell, sell_token_decimals);

    let is_sell = response.kind == "sell";
    let (net_surplus, surplus_percentage) = if is_sell {
        let net = executed_buy.parse::<f64>().unwrap() - buy.parse::<f64>().unwrap();
        let percentage = format_into_four_decimal_point(net / buy.parse::<f64>().unwrap());
        (format_into_four_decimal_point(net), percentage)
    } else {
        let net = sell.parse::<f64>().unwrap() - executed_sell.parse::<f64>().unwrap();
        let percentage = format_into_four_decimal_point(net / sell.parse::<f64>().unwrap());
        (format_into_four_decimal_point(net), percentage)
    };

    Order {
        uid,
        is_sell,
        buy,
        sell,
        executed_buy,
        executed_sell,
        net_surplus,
        surplus_percentage,
    }
}
