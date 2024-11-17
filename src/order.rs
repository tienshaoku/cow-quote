use crate::format::format_into_four_decimal_point;

#[derive(Debug)]
pub struct Order {
    uid: String,
    is_sell: bool,
    buy: String,
    sell: String,
    executed_buy: String,
    executed_sell: String,
    net_surplus: String,
    surplus_percentage: String,
}

impl Order {
    pub fn from(uid: String, is_sell: bool) -> Self {
        Order {
            uid,
            is_sell,
            buy: String::new(),
            sell: String::new(),
            executed_buy: String::new(),
            executed_sell: String::new(),
            net_surplus: String::new(),
            surplus_percentage: String::new(),
        }
    }

    pub fn update_surplus(
        &mut self,
        executed_buy: &str,
        executed_sell: &str,
        buy: &str,
        sell: &str,
        is_sell: bool,
    ) {
        let mut net_surplus: f64 = 0.0;
        let mut surplus_percentage: String;

        if is_sell {
            net_surplus = executed_buy.parse::<f64>().unwrap() - buy.parse::<f64>().unwrap();
            surplus_percentage =
                format_into_four_decimal_point(net_surplus / buy.parse::<f64>().unwrap());
        } else {
            net_surplus = sell.parse::<f64>().unwrap() - executed_sell.parse::<f64>().unwrap();
            surplus_percentage =
                format_into_four_decimal_point(net_surplus / sell.parse::<f64>().unwrap());
        }

        self.is_sell = is_sell;
        self.buy = buy.to_string();
        self.sell = sell.to_string();
        self.executed_buy = executed_buy.to_string();
        self.executed_sell = executed_sell.to_string();
        self.net_surplus = format_into_four_decimal_point(net_surplus);
        self.surplus_percentage = surplus_percentage;
    }
}
