use crate::helper::EnvConfig;
use crate::order::Order;
use aws_sdk_dynamodb::{operation::get_item::GetItemOutput, types::AttributeValue, Client, Error};
use ethers::{
    middleware::Middleware,
    providers::{Http, Provider},
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct DynamoDbClient {
    client: Client,
    table_name: String,
}

impl DynamoDbClient {
    pub async fn new() -> eyre::Result<Self> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            table_name: "orders".to_string(),
        })
    }

    pub async fn upload_order(&self, order: &Order) -> eyre::Result<()> {
        let item: HashMap<String, AttributeValue> = HashMap::from([
            ("uid".to_string(), to_attr(order.uid(), "S")),
            ("owner".to_string(), to_attr(order.owner(), "S")),
            ("buy_token".to_string(), to_attr(order.buy_token(), "S")),
            ("sell_token".to_string(), to_attr(order.sell_token(), "S")),
            (
                "buy_decimals".to_string(),
                to_attr(order.buy_decimals(), "N"),
            ),
            (
                "sell_decimals".to_string(),
                to_attr(order.sell_decimals(), "N"),
            ),
            ("min_buy".to_string(), to_attr(order.min_buy(), "N")),
            ("sell".to_string(), to_attr(order.sell(), "N")),
            (
                "executed_buy".to_string(),
                to_attr(order.executed_buy(), "N"),
            ),
            (
                "executed_sell".to_string(),
                to_attr(order.executed_sell(), "N"),
            ),
            ("net_surplus".to_string(), to_attr(order.net_surplus(), "N")),
            (
                "surplus_percentage".to_string(),
                to_attr(order.surplus_percentage(), "N"),
            ),
            (
                "zerox_quote_buy".to_string(),
                to_attr(order.zerox_quote_buy(), "N"),
            ),
            (
                "compared_executed_with_zerox_quote".to_string(),
                to_attr(order.compared_executed_with_zerox_quote(), "N"),
            ),
            (
                "compared_with_zerox_percentage".to_string(),
                to_attr(order.compared_with_zerox_percentage(), "N"),
            ),
            (
                "cows_own_quote_buy".to_string(),
                to_attr(order.cows_own_quote_buy(), "N"),
            ),
            (
                "compared_executed_with_cows_own_quote".to_string(),
                to_attr(order.compared_executed_with_cows_own_quote(), "N"),
            ),
            (
                "compared_with_cows_own_quote_percentage".to_string(),
                to_attr(order.compared_with_cows_own_quote_percentage(), "N"),
            ),
            (
                "univ3_swap_buy".to_string(),
                to_attr(order.univ3_swap_buy(), "N"),
            ),
            (
                "compared_executed_with_univ3_swap".to_string(),
                to_attr(order.compared_executed_with_univ3_swap(), "N"),
            ),
            (
                "compared_with_univ3_swap_percentage".to_string(),
                to_attr(order.compared_with_univ3_swap_percentage(), "N"),
            ),
            (
                "block_number".to_string(),
                to_attr(order.block_number(), "N"),
            ),
            ("timestamp".to_string(), to_attr(order.timestamp(), "N")),
        ]);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_item(
        &self,
        table_name: &str,
        key: &str,
        value: AttributeValue,
    ) -> eyre::Result<Option<HashMap<String, AttributeValue>>> {
        let result: GetItemOutput = self
            .client
            .get_item()
            .table_name(table_name)
            .key(key, value)
            .send()
            .await?;

        Ok(result.item)
    }

    pub async fn get_items_with_timestamp(
        &self,
        table_name: &str,
        key: &str,
        timestamp: AttributeValue,
    ) -> eyre::Result<Vec<HashMap<String, AttributeValue>>> {
        let result = self
            .client
            .scan()
            .table_name(table_name)
            .filter_expression(format!("{} > :val", key))
            .expression_attribute_values(":val", timestamp)
            .send()
            .await?;

        if result.items.is_none() || result.items.as_ref().unwrap().is_empty() {
            return Err(eyre::eyre!("No items found in the result"));
        }
        Ok(result.items.unwrap())
    }
}

fn to_attr(value: impl ToString, attr_type: &str) -> AttributeValue {
    match attr_type {
        "N" => AttributeValue::N(value.to_string()),
        _ => AttributeValue::S(value.to_string()),
    }
}

pub async fn fetch_latest_from_database(config: &EnvConfig) -> eyre::Result<Vec<Order>> {
    let client = DynamoDbClient::new().await?;
    let provider = Provider::<Http>::try_from(config.get_alchemy_http_url())?;

    let table_name = "orders";
    let key = "block_number";
    let block_number = provider.get_block_number().await?;
    let value = AttributeValue::N((block_number - 15).to_string());

    let items = client
        .get_items_with_timestamp(table_name, key, value)
        .await?;

    let mut orders = Vec::new();
    for item in items {
        let order = Order::from_dynamodb_item(&item);
        println!("Get Order: {:?}", order);
        orders.push(order);
    }
    Ok(orders)
}

pub fn extract_number(item: &HashMap<String, AttributeValue>, key: &str) -> u64 {
    item.get(key)
        .and_then(|v| v.as_n().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or_default()
}

pub fn extract_string(item: &HashMap<String, AttributeValue>, key: &str) -> String {
    item.get(key)
        .and_then(|v| v.as_s().ok())
        .map(|v| v.to_string()) // &String to String
        .unwrap_or_else(|| String::new())
}
