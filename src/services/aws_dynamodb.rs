use crate::order::Order;
use aws_sdk_dynamodb::{operation::get_item::GetItemOutput, types::AttributeValue, Client, Error};
use std::collections::HashMap;

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
}

fn to_attr(value: impl ToString, attr_type: &str) -> AttributeValue {
    match attr_type {
        "N" => AttributeValue::N(value.to_string()),
        _ => AttributeValue::S(value.to_string()),
    }
}

// TODO:
// 1. pass client to this function
// 2. pass key and value
pub async fn fetch_latest_from_database() -> eyre::Result<Order> {
    let client = DynamoDbClient::new().await?;

    let table_name = "orders";
    let key = "uid";
    let value = AttributeValue::S("0x39f456b902d7fab8becf74bee9e9568ac33f6f7fc6b6cfc37ee34e92adbb2907e5c0830d260bf94c994a2392b7c8d8f2a593f3576759c2b6".to_string());

    let result = client.get_item(table_name, key, value).await?;

    if let Some(item) = result {
        let order = Order::from_dynamodb_item(&item);
        println!("Get Order: {:?}", order);
        Ok(order)
    } else {
        Err(eyre::eyre!("Item not found"))
    }
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
