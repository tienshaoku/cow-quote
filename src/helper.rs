use ethers::{types::U256, utils::format_units};

pub fn format_decimals_into_f(amount: &str, decimals: u8) -> f64 {
    let formatted = format_units(U256::from_dec_str(amount).unwrap(), decimals as u32).unwrap();
    formatted.parse::<f64>().unwrap()
}
