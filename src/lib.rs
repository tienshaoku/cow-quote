mod constant;

use std::sync::Arc;

use ethers::{
    contract::{Contract, EthEvent},
    providers::{Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
#[ethevent(name = "Trade")]
pub struct Trade {
    #[ethevent(indexed)]
    pub owner: Address,
    #[serde(rename = "sellToken")]
    pub sell_token: Address,
    #[serde(rename = "buyToken")]
    pub buy_token: Address,
    #[serde(rename = "sellAmount")]
    pub sell_amount: U256,
    #[serde(rename = "buyAmount")]
    pub buy_amount: U256,
    #[serde(rename = "feeAmount")]
    pub fee_amount: U256,
    #[serde(rename = "orderUid")]
    pub order_uid: Bytes,
}

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
pub struct Settlement {
    #[ethevent(indexed)]
    pub solver: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
#[ethevent(name = "Trade")]
pub struct Interaction {
    #[ethevent(indexed)]
    pub target: Address,
    #[serde(rename = "sellToken")]
    pub value: U256,
    #[serde(rename = "buyToken")]
    pub selector: [u8; 4],
}

use futures::StreamExt;

pub async fn run() -> eyre::Result<()> {
    let provider = Provider::<Ws>::connect(constant::ETH_RPC).await?;
    let provider = Arc::new(provider);

    let contract_address = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41".parse::<Address>()?;
    let filter = Filter::new().address(contract_address);

    let event = Trade::new::<_, Provider<Ws>>(filter, Arc::clone(&provider));
    let mut transfers = event.subscribe().await?.take(1);
    while let Some(log) = transfers.next().await {
        println!("Transfer: {:?}", log);
    }

    Ok(())
}
