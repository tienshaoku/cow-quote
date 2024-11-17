use crate::secret;
use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct ZeroXResponse {
    pub buy_amount: String,
    pub min_buy_amount: String,
    pub sources: Vec<String>,
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
    println!("{:#?}", response);

    // Extract all sources from the fills array
    let empty_vec = Vec::new();
    let fills = response["route"]["fills"].as_array().unwrap_or(&empty_vec);
    let sources: Vec<String> = fills
        .iter()
        .filter_map(|fill: &serde_json::Value| fill["source"].as_str().map(String::from))
        .collect();

    Ok(ZeroXResponse {
        buy_amount: extract_string_from_value(&response, "buyAmount"),
        min_buy_amount: extract_string_from_value(&response, "minBuyAmount"),
        sources,
    })
}

fn extract_string_from_value(value: &serde_json::Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_string()
}
