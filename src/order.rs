use crate::constant;
use crate::contract::ierc20::get_token_decimals;
use crate::helper::format_decimals_into_f;
use crate::services::cow_get_order_api::CowGetResponse;
use getset::Getters;

use ethers::{
    providers::{Provider, Ws},
    types::Address,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize, Clone, Default, Getters)]
#[getset(get = "pub")]
pub struct Order {
    uid: String,
    owner: String,

    buy_token: String,
    sell_token: String,
    buy_decimals: u8,
    sell_decimals: u8,

    min_buy: f64,
    sell: f64,
    executed_buy: f64,
    executed_sell: f64,

    net_surplus: f64,
    surplus_percentage: f64,

    zerox_quote_buy: f64,
    compared_executed_with_zerox_quote: f64,
    compared_with_zerox_percentage: f64,

    cows_own_quote_buy: f64,
    compared_executed_with_cows_own_quote: f64,
    compared_with_cows_own_quote_percentage: f64,

    univ3_swap_buy: f64,
    compared_executed_with_univ3_swap: f64,
    compared_with_univ3_swap_percentage: f64,

    block_number: u64,
    // in unlikely cases where timestamp == 0, data user can query it from block_number
    timestamp: u64,
}

impl Order {
    pub async fn from_cow_api_response(
        provider: Arc<Provider<Ws>>,
        uid: String,
        block_number: u64,
        timestamp: u64,
        response: &CowGetResponse,
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

        let net_surplus = executed_buy - min_buy;
        let surplus_percentage = net_surplus / min_buy;

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
            net_surplus,
            surplus_percentage,
            block_number,
            timestamp,
            ..Default::default()
        })
    }

    fn calculate_percentage(&self, input: f64) -> f64 {
        if input == 0.0 {
            1.0
        } else {
            let denominator = if self.min_buy == 0.0 {
                &self.executed_buy
            } else {
                &self.min_buy
            };
            self.compare(input) / denominator
        }
    }

    fn compare(&self, input: f64) -> f64 {
        self.executed_buy - input
    }

    pub fn update_zerox_comparison(&mut self, quote_buy: &str) {
        self.zerox_quote_buy = format_decimals_into_f(quote_buy, self.buy_decimals);
        self.compared_executed_with_zerox_quote = self.compare(self.zerox_quote_buy);
        self.compared_with_zerox_percentage = self.calculate_percentage(self.zerox_quote_buy);
    }

    pub fn update_cows_own_quote_comparison(&mut self, quote_buy: &str) {
        self.cows_own_quote_buy = format_decimals_into_f(quote_buy, self.buy_decimals);
        self.compared_executed_with_cows_own_quote = self.compare(self.cows_own_quote_buy);
        self.compared_with_cows_own_quote_percentage =
            self.calculate_percentage(self.cows_own_quote_buy);
    }

    pub fn update_univ3_swap_comparison(&mut self, quote_buy: &str) {
        self.univ3_swap_buy = format_decimals_into_f(quote_buy, self.buy_decimals);
        self.compared_executed_with_univ3_swap = self.compare(self.univ3_swap_buy);
        self.compared_with_univ3_swap_percentage = self.calculate_percentage(self.univ3_swap_buy);
    }

    pub fn no_successful_quote_at_all(&self) -> bool {
        self.zerox_quote_buy == 0.0
            && self.compared_executed_with_zerox_quote == 0.0
            && self.compared_with_zerox_percentage == 0.0
            && self.cows_own_quote_buy == 0.0
            && self.compared_executed_with_cows_own_quote == 0.0
            && self.compared_with_cows_own_quote_percentage == 0.0
            && self.univ3_swap_buy == 0.0
            && self.compared_executed_with_univ3_swap == 0.0
            && self.compared_with_univ3_swap_percentage == 0.0
    }
}

async fn process_order_info(
    provider: Arc<Provider<Ws>>,
    address: &str,
    planned_amount: &str,
    executed_amount: &str,
) -> eyre::Result<(u8, f64, f64)> {
    let is_weth = address == constant::WETH;
    let decimals =
        get_token_decimals(Arc::clone(&provider), address.parse::<Address>()?, is_weth).await?;

    let planned = format_decimals_into_f(planned_amount, decimals);
    let executed = format_decimals_into_f(executed_amount, decimals);

    Ok((decimals, planned, executed))
}
