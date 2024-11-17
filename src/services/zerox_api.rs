use crate::secret;
use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct ZeroXResponse {
    pub buy: String,
    pub min_buy: String,
    pub sources: Vec<String>,
}

impl ZeroXResponse {
    pub fn buy(&self) -> &str {
        &self.buy
    }

    pub fn min_buy(&self) -> &str {
        &self.min_buy
    }

    pub fn sources(&self) -> &Vec<String> {
        &self.sources
    }

    pub fn is_empty(&self) -> bool {
        // cast then compare to avoid empty string with many 0 suffixes
        self.buy.parse::<f64>().unwrap_or(0.0) == 0.0
            && self.min_buy.parse::<f64>().unwrap_or(0.0) == 0.0
            && self.sources.is_empty()
    }

    fn from_empty() -> Self {
        Self {
            buy: "0".to_string(),
            min_buy: "0".to_string(),
            sources: vec![],
        }
    }
}

pub async fn get_zerox_price_quote(
    chain_id: &str,
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
    taker_address: &str,
) -> Result<ZeroXResponse, reqwest::Error> {
    let client = reqwest::Client::new();

    let params = HashMap::from([
        ("chainId", chain_id),
        ("sellToken", sell_token),
        ("buyToken", buy_token),
        ("sellAmount", sell_amount),
        ("takerAddress", taker_address),
    ]);

    let mut headers = HeaderMap::new();
    headers.insert(
        "0x-api-key",
        HeaderValue::from_static(secret::ZEROX_API_KEY),
    );
    headers.insert("0x-version", HeaderValue::from_static("v2"));

    let response: serde_json::Value = client
        .get("https://api.0x.org/swap/permit2/price?")
        .headers(headers)
        .query(&params)
        .send()
        .await?
        .json()
        .await?;
    // println!("{:#?}", response);

    // Extract all sources from the fills array
    let empty_vec = Vec::new();
    let fills = response["route"]["fills"].as_array().unwrap_or(&empty_vec);
    let sources: Vec<String> = fills
        .iter()
        .filter_map(|fill: &serde_json::Value| fill["source"].as_str().map(String::from))
        .collect();

    let mut response = ZeroXResponse {
        buy: extract_string_from_value(&response, "buyAmount"),
        min_buy: extract_string_from_value(&response, "minBuyAmount"),
        sources,
    };

    if response.is_empty() {
        response = ZeroXResponse::from_empty();
    }

    Ok(response)
}

fn extract_string_from_value(value: &serde_json::Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_string()
}
