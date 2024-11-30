use crate::secret;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ZeroXGetResponse {
    #[serde(rename = "buyAmount")]
    buy: String,
    #[serde(rename = "totalNetworkFee")]
    total_network_fee: String,
    #[serde(rename = "liquidityAvailable")]
    liquidity_available: bool,
    route: Route,
    #[serde(skip)]
    _others: (),
}

impl ZeroXGetResponse {
    pub fn is_invalid(&self) -> bool {
        !self.liquidity_available
    }
}

#[derive(Debug, Deserialize)]
struct Route {
    fills: Vec<Fill>,
    #[serde(skip)]
    _others: (),
}

#[derive(Debug, Deserialize)]
struct Fill {
    source: String,
    #[serde(skip)]
    _others: (),
}

pub async fn zerox_quote_buy(
    client: &reqwest::Client,
    chain_id: &str,
    taker_address: &str,
    sell_token: &str,
    buy_token: &str,
    sell: &str,
) -> eyre::Result<String> {
    let params = HashMap::from([
        ("chainId", chain_id),
        ("sellToken", sell_token),
        ("buyToken", buy_token),
        ("sellAmount", sell),
        ("takerAddress", taker_address),
    ]);

    let mut headers = HeaderMap::new();
    headers.insert(
        "0x-api-key",
        HeaderValue::from_static(secret::ZEROX_API_KEY),
    );
    headers.insert("0x-version", HeaderValue::from_static("v2"));

    let zerox_response: ZeroXGetResponse = client
        .get("https://api.0x.org/swap/permit2/price?")
        .headers(headers)
        .query(&params)
        .send()
        .await
        .map_err(|e| eyre::eyre!("Failed to send request: {}", e))?
        .json()
        .await
        .map_err(|e| eyre::eyre!("Failed to parse response into json: {}", e))?;

    if zerox_response.is_invalid() {
        Err(eyre::eyre!("0x liquidity not available"))
    } else {
        Ok(zerox_response.buy)
    }
}
