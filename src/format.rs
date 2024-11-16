use ethers::{types::U256, utils::format_units};

fn trim_decimal_point(amount: &str) -> String {
    if amount.ends_with(".0000") {
        amount.trim_end_matches(".0000").to_string()
    } else {
        amount.to_string()
    }
}

pub fn format_into_four_decimal_point<T: std::fmt::Display>(amount: T) -> String {
    let formatted = format!("{:.4}", amount);
    trim_decimal_point(&formatted)
}

pub fn format_decimals(amount: &str, decimals: u32) -> String {
    let formatted = format!(
        "{:.*}",
        18_usize,
        format_units(U256::from_dec_str(amount).unwrap(), decimals).unwrap()
    );
    trim_decimal_point(&formatted)
}
