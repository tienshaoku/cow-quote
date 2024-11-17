use crate::constant;
use crate::format::{format_decimal_point, format_decimals, format_four_decimal_point};
use crate::ierc20::get_token_decimals;
use crate::services::cow_api::CowAPIResponse;
use crate::services::zerox_api::ZeroXResponse;

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

    min_buy: String,
    sell: String,
    executed_buy: String,
    executed_sell: String,

    net_surplus: String,
    surplus_percentage: String,

    zerox_quote_buy: String,
    zerox_min_buy: String,
    zerox_sources: Vec<String>,
    compared_executed_with_zerox_quote: String,
    compared_min_buy: String,
    compared_with_zerox_percentage: String,

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
        let (buy_decimals, min_buy, executed_buy) = process_order_info(
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

        net_surplus = executed_buy.parse::<f64>().unwrap() - min_buy.parse::<f64>().unwrap();
        surplus_percentage =
            format_four_decimal_point(net_surplus / min_buy.parse::<f64>().unwrap());

        Ok(Order {
            uid,
            owner: response.owner().to_string(),
            buy_token: response.buy_token().to_string(),
            sell_token: response.sell_token().to_string(),
            buy_decimals,
            sell_decimals,
            min_buy,
            sell,
            executed_buy,
            executed_sell,
            net_surplus: format_decimal_point(net_surplus, buy_decimals),
            surplus_percentage,
            block_number,
            timestamp,
            zerox_quote_buy: "".to_string(),
            zerox_min_buy: "".to_string(),
            zerox_sources: vec![],
            compared_executed_with_zerox_quote: "".to_string(),
            compared_min_buy: "".to_string(),
            compared_with_zerox_percentage: "".to_string(),
        })
    }

    pub fn update_zerox_comparison(&mut self, response: ZeroXResponse) {
        let decimals: u8 = self.buy_decimals;

        let zerox_quote_buy = format_decimals(response.buy(), decimals);
        let zerox_min_buy = format_decimals(response.min_buy(), decimals);

        let compared_executed_with_zerox_quote =
            compare_with_zerox(&self.executed_buy, &zerox_quote_buy, decimals);

        self.zerox_quote_buy = zerox_quote_buy;
        self.zerox_min_buy = zerox_min_buy.clone();
        self.zerox_sources = response.sources().to_vec();

        self.compared_executed_with_zerox_quote = compared_executed_with_zerox_quote.clone();
        self.compared_min_buy = compare_with_zerox(&self.min_buy, &zerox_min_buy, decimals);
        self.compared_with_zerox_percentage = if response.is_empty() {
            "1".to_string()
        } else {
            format_four_decimal_point(
                &compared_executed_with_zerox_quote.parse::<f64>().unwrap()
                    / self.min_buy.parse::<f64>().unwrap(),
            )
        };
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

fn compare_with_zerox(cow: &str, zerox: &str, decimals: u8) -> String {
    format_decimal_point(
        cow.parse::<f64>().unwrap() - zerox.parse::<f64>().unwrap(),
        decimals,
    )
}
