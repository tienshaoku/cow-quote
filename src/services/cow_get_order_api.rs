use getset::Getters;
use reqwest;
use serde::Deserialize;

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
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
