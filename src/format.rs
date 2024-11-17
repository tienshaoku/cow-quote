use ethers::{types::U256, utils::format_units};

fn trim_decimal_point(amount: &str) -> String {
    amount
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

pub fn format_decimal_point<T: std::fmt::Display>(amount: T, decimals: u8) -> String {
    let formatted: String = format!("{:.*}", decimals as usize, amount);
    trim_decimal_point(&formatted)
}

pub fn format_four_decimal_point<T: std::fmt::Display>(amount: T) -> String {
    let formatted: String = format!("{:.4}", amount);
    trim_decimal_point(&formatted)
}

pub fn format_decimals(amount: &str, decimals: u8) -> String {
    let formatted = format_units(U256::from_dec_str(amount).unwrap(), decimals as u32).unwrap();
    format_decimal_point(formatted, decimals)
}
