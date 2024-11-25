use crate::order::Order;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use std::collections::HashMap;

pub struct AwsClient {
    client: Client,
    table_name: String,
}

impl AwsClient {
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

    fn to_attr(value: impl ToString, attr_type: &str) -> AttributeValue {
        match attr_type {
            "N" => AttributeValue::N(value.to_string()),
            _ => AttributeValue::S(value.to_string()),
        }
    }

    pub async fn upload_order(&self, order: &Order) -> eyre::Result<()> {
        let item: HashMap<String, AttributeValue> = HashMap::from([
            ("uid".to_string(), Self::to_attr(order.uid(), "S")),
            ("owner".to_string(), Self::to_attr(order.owner(), "S")),
            (
                "buy_token".to_string(),
                Self::to_attr(order.buy_token(), "S"),
            ),
            (
                "sell_token".to_string(),
                Self::to_attr(order.sell_token(), "S"),
            ),
            (
                "buy_decimals".to_string(),
                Self::to_attr(order.buy_decimals(), "N"),
            ),
            (
                "sell_decimals".to_string(),
                Self::to_attr(order.sell_decimals(), "N"),
            ),
            ("min_buy".to_string(), Self::to_attr(order.min_buy(), "N")),
            ("sell".to_string(), Self::to_attr(order.sell(), "N")),
            (
                "executed_buy".to_string(),
                Self::to_attr(order.executed_buy(), "N"),
            ),
            (
                "executed_sell".to_string(),
                Self::to_attr(order.executed_sell(), "N"),
            ),
            (
                "net_surplus".to_string(),
                Self::to_attr(order.net_surplus(), "N"),
            ),
            (
                "surplus_percentage".to_string(),
                Self::to_attr(order.surplus_percentage(), "N"),
            ),
            (
                "zerox_quote_buy".to_string(),
                Self::to_attr(order.zerox_quote_buy(), "N"),
            ),
            (
                "zerox_min_buy".to_string(),
                Self::to_attr(order.zerox_min_buy(), "N"),
            ),
            (
                "zerox_sources".to_string(),
                AttributeValue::L(
                    order
                        .zerox_sources()
                        .iter()
                        .map(|s| Self::to_attr(s, "S"))
                        .collect(),
                ),
            ),
            (
                "compared_min_buy".to_string(),
                Self::to_attr(order.compared_min_buy(), "N"),
            ),
            (
                "compared_executed_with_zerox_quote".to_string(),
                Self::to_attr(order.compared_executed_with_zerox_quote(), "N"),
            ),
            (
                "compared_with_zerox_percentage".to_string(),
                Self::to_attr(order.compared_with_zerox_percentage(), "N"),
            ),
            (
                "cows_own_quote_buy".to_string(),
                Self::to_attr(order.cows_own_quote_buy(), "N"),
            ),
            (
                "compared_executed_with_cows_own_quote".to_string(),
                Self::to_attr(order.compared_executed_with_cows_own_quote(), "N"),
            ),
            (
                "compared_with_cows_own_quote_percentage".to_string(),
                Self::to_attr(order.compared_with_cows_own_quote_percentage(), "N"),
            ),
            (
                "univ3_swap_buy".to_string(),
                Self::to_attr(order.univ3_swap_buy(), "N"),
            ),
            (
                "compared_executed_with_univ3_swap".to_string(),
                Self::to_attr(order.compared_executed_with_univ3_swap(), "N"),
            ),
            (
                "compared_with_univ3_swap_percentage".to_string(),
                Self::to_attr(order.compared_with_univ3_swap_percentage(), "N"),
            ),
            (
                "block_number".to_string(),
                Self::to_attr(order.block_number(), "N"),
            ),
            (
                "timestamp".to_string(),
                Self::to_attr(order.timestamp(), "N"),
            ),
        ]);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;

        Ok(())
    }
}
