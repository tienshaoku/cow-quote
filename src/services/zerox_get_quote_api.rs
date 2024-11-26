use crate::secret;
use getset::Getters;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ZeroXGetResponse {
    #[serde(rename = "buyAmount")]
    buy: String,
    #[serde(rename = "minBuyAmount")]
    min_buy: String,
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
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ZeroXResponse {
    pub is_empty: bool,
    pub buy: String,
    pub min_buy: String,
    pub sources: Vec<String>,
    pub total_network_fee: String,
    pub liquidity_available: bool,
}

impl ZeroXResponse {
    fn from_empty() -> Self {
        Self {
            is_empty: true,
            buy: "0".to_string(),
            min_buy: "0".to_string(),
            sources: vec![],
            total_network_fee: "0".to_string(),
            liquidity_available: false,
        }
    }
}

async fn get_zerox_response(
    chain_id: &str,
    sell_token: &str,
    buy_token: &str,
    sell: &str,
    taker_address: &str,
) -> Result<ZeroXGetResponse, reqwest::Error> {
    let client = reqwest::Client::new();
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

    client
        .get("https://api.0x.org/swap/permit2/price?")
        .headers(headers)
        .query(&params)
        .send()
        .await?
        .json()
        .await
}

pub async fn zerox_get_quote(
    chain_id: &str,
    sell_token: &str,
    buy_token: &str,
    sell: &str,
    taker_address: &str,
) -> eyre::Result<ZeroXResponse> {
    let zerox_response =
        get_zerox_response(chain_id, sell_token, buy_token, sell, taker_address).await?;

    let sources: Vec<String> = zerox_response
        .route
        .fills
        .iter()
        .map(|fill| fill.source.clone())
        .collect();

    let response = if zerox_response.is_invalid() {
        ZeroXResponse::from_empty()
    } else {
        ZeroXResponse {
            is_empty: false,
            buy: zerox_response.buy,
            min_buy: zerox_response.min_buy,
            sources,
            total_network_fee: zerox_response.total_network_fee,
            liquidity_available: zerox_response.liquidity_available,
        }
    };

    Ok(response)
}
