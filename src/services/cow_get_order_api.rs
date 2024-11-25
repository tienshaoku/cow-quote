use reqwest;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowGetResponse {
    owner: String,
    buy_token: String,
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

impl CowGetResponse {
    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn buy_token(&self) -> &str {
        &self.buy_token
    }

    pub fn sell_token(&self) -> &str {
        &self.sell_token
    }

    pub fn buy(&self) -> &str {
        &self.buy
    }

    pub fn sell(&self) -> &str {
        &self.sell
    }

    pub fn executed_buy(&self) -> &str {
        &self.executed_buy
    }

    pub fn executed_sell(&self) -> &str {
        &self.executed_sell
    }

    pub fn is_sell(&self) -> bool {
        self.kind == "sell"
    }
}

pub async fn cowswap_get_order(order_uid: &str) -> Result<(CowGetResponse, bool), reqwest::Error> {
    let url: String = format!("https://api.cow.fi/mainnet/api/v1/orders/{}", order_uid);
    let response = reqwest::get(&url).await?.json::<CowGetResponse>().await?;

    // 1. 0x has only sell orders
    // 2. exclude partial fills for now
    let should_proceed = response.is_sell() && response.sell() == response.executed_sell();
    Ok((response, should_proceed))
}
