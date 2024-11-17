use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct CowQuoteRequest {
    #[serde(rename = "sellToken")]
    sell_token: String,
    #[serde(rename = "buyToken")]
    buy_token: String,
    receiver: String,
    #[serde(rename = "appData")]
    app_data: String,
    #[serde(rename = "appDataHash")]
    app_data_hash: String,
    #[serde(rename = "sellTokenBalance")]
    sell_token_balance: String,
    #[serde(rename = "buyTokenBalance")]
    buy_token_balance: String,
    from: String,
    #[serde(rename = "priceQuality")]
    price_quality: String,
    #[serde(rename = "signingScheme")]
    signing_scheme: String,
    #[serde(rename = "onchainOrder")]
    onchain_order: bool,
    kind: String,
    #[serde(rename = "sellAmountBeforeFee")]
    sell_amount_before_fee: String,
}

#[derive(Debug, Deserialize)]
struct CowQuoteResponse {
    quote: Quote,
    #[serde(skip)]
    _others: (),
}

#[derive(Debug, Deserialize)]
struct Quote {
    #[serde(rename = "sellAmount")]
    sell: String,
    #[serde(rename = "buyAmount")]
    buy: String,
    #[serde(skip)]
    _others: (),
}

pub async fn post_cowswap_quote(
    owner: &str,
    sell_token: &str,
    buy_token: &str,
    sell: &str,
) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = "https://api.cow.fi/mainnet/api/v1/quote";

    let quote_request = CowQuoteRequest {
        sell_token: sell_token.to_string(),
        buy_token: buy_token.to_string(),
        receiver: owner.to_string(),
        app_data: "{\"version\":\"0.9.0\",\"metadata\":{}}".to_string(),
        app_data_hash: "0xc990bae86208bfdfba8879b64ab68da5905e8bb97aa3da5c701ec1183317a6f6"
            .to_string(),
        from: owner.to_string(),
        sell_token_balance: "erc20".to_string(),
        buy_token_balance: "erc20".to_string(),
        price_quality: "verified".to_string(),
        signing_scheme: "eip712".to_string(),
        onchain_order: false,
        kind: "sell".to_string(),
        sell_amount_before_fee: sell.to_string(),
    };

    let response: CowQuoteResponse = client
        .post(url)
        .json(&quote_request)
        .send()
        .await?
        .json()
        .await?;

    assert_eq!(response.quote.sell, sell);
    Ok(response.quote.buy)
}
