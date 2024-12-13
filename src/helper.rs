use dotenv::dotenv;
use ethers::{types::U256, utils::format_units};
use getset::Getters;
use std::env;

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct EnvConfig {
    alchemy_rpc_url: String,
    zerox_api_key: String,
}

impl EnvConfig {
    pub fn new() -> Self {
        dotenv().ok();

        let alchemy_rpc_url = "ALCHEMY_RPC_URL";
        let zerox_api_key = "ZEROX_API_KEY";
        let error_message = "must be set in .env";
        Self {
            alchemy_rpc_url: env::var(alchemy_rpc_url)
                .expect(&format!("{} {}", alchemy_rpc_url, error_message)),
            zerox_api_key: env::var(zerox_api_key)
                .expect(&format!("{} {}", zerox_api_key, error_message)),
        }
    }

    pub fn get_alchemy_http_url(&self) -> String {
        format!("https://{}", self.alchemy_rpc_url)
    }

    pub fn get_alchemy_wss_url(&self) -> String {
        format!("wss://{}", self.alchemy_rpc_url)
    }
}

pub fn format_decimals_into_f(amount: &str, decimals: u8) -> f64 {
    let formatted = format_units(U256::from_dec_str(amount).unwrap(), decimals as u32).unwrap();
    formatted.parse::<f64>().unwrap()
}
