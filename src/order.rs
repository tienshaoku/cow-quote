use crate::constant;
use crate::format::{format_decimals, format_into_four_decimal_point};
use crate::ierc20::get_token_decimals;
use crate::services::cow_api::CowAPIResponse;

use ethers::{
    providers::{Provider, Ws},
    types::Address,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    uid: String,
    owner: String,

    buy_token: String,
    sell_token: String,
    buy_decimals: u8,
    sell_decimals: u8,

    buy: String,
    sell: String,
    executed_buy: String,
    executed_sell: String,

    net_surplus: String,
    surplus_percentage: String,

    block_number: u64,
    timestamp: u64,
}

impl Order {
    pub async fn from_cow_api_response(
        provider: Arc<Provider<Ws>>,
        uid: String,
        block_number: u64,
        timestamp: u64,
        response: &CowAPIResponse,
    ) -> eyre::Result<Self> {
        let (buy_decimals, buy, executed_buy) = process_order_info(
            Arc::clone(&provider),
            &response.buy_token(),
            &response.buy(),
            &response.executed_buy(),
        )
        .await?;

        let (sell_decimals, sell, executed_sell) = process_order_info(
            Arc::clone(&provider),
            &response.sell_token(),
            &response.sell(),
            &response.executed_sell(),
        )
        .await?;

        let mut net_surplus: f64 = 0.0;
        let mut surplus_percentage: String;

        if response.is_sell() {
            net_surplus = executed_buy.parse::<f64>().unwrap() - buy.parse::<f64>().unwrap();
            surplus_percentage =
                format_into_four_decimal_point(net_surplus / buy.parse::<f64>().unwrap());
        } else {
            net_surplus = sell.parse::<f64>().unwrap() - executed_sell.parse::<f64>().unwrap();
            surplus_percentage =
                format_into_four_decimal_point(net_surplus / sell.parse::<f64>().unwrap());
        }

        Ok(Order {
            uid,
            owner: response.owner().to_string(),
            buy_token: response.buy_token().to_string(),
            sell_token: response.sell_token().to_string(),
            buy_decimals,
            sell_decimals,
            buy,
            sell,
            executed_buy,
            executed_sell,
            net_surplus: format_into_four_decimal_point(net_surplus),
            surplus_percentage,
            block_number,
            timestamp,
        })
    }
}

async fn process_order_info(
    provider: Arc<Provider<Ws>>,
    address: &str,
    planned_amount: &str,
    executed_amount: &str,
) -> eyre::Result<(u8, String, String)> {
    let is_weth = address == constant::WETH;
    let decimals =
        get_token_decimals(Arc::clone(&provider), address.parse::<Address>()?, is_weth).await?;

    let planned = format_decimals(planned_amount, decimals);
    let executed = format_decimals(executed_amount, decimals);

    Ok((decimals, planned, executed))
}
