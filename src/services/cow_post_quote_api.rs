use ethers::types::U256;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteParam {
    sell_token: String,
    buy_token: String,
    receiver: String,
    app_data: String,
    app_data_hash: String,
    sell_token_balance: String,
    buy_token_balance: String,
    from: String,
    price_quality: String,
    signing_scheme: String,
    onchain_order: bool,
    kind: String,
    sell_amount_before_fee: String,
}

#[derive(Debug, Deserialize)]
struct CowPostResponse {
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
    #[serde(rename = "feeAmount")]
    fee: String,
    #[serde(skip)]
    _others: (),
}

pub async fn cowswap_quote_buy(
    client: &reqwest::Client,
    owner: &str,
    sell_token: &str,
    buy_token: &str,
    sell: &str,
) -> eyre::Result<String> {
    let url = "https://api.cow.fi/mainnet/api/v1/quote";

    let quote_param = QuoteParam {
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

    let response: CowPostResponse = client
        .post(url)
        .json(&quote_param)
        .send()
        .await
        .map_err(|e| eyre::eyre!("Failed to send request: {}", e))?
        .json()
        .await
        .map_err(|e| eyre::eyre!("Failed to parse response into json: {}", e))?;

    // response.quote.sell & sell aren't always the same because of fees
    let original_sell = U256::from_dec_str(sell).unwrap();
    let sell = U256::from_dec_str(&response.quote.sell).unwrap();
    let fee = U256::from_dec_str(&response.quote.fee).unwrap();

    match sell == original_sell || sell + fee == original_sell {
        true => Ok(response.quote.buy),
        false => Err(eyre::eyre!("Sell amount mismatch")),
    }
}
