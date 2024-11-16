#[path = "constant.rs"]
mod constant;
mod secret;

use std::sync::Arc;

use ethers::{
    contract::EthEvent,
    providers::{Provider, Ws},
    types::{Address, Bytes, Filter, ValueOrArray, U256},
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
#[ethevent(name = "Trade")]
pub struct TradeEvent {
    #[ethevent(indexed)]
    pub owner: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: U256,
    pub buy_amount: U256,
    pub fee_amount: U256,
    pub order_uid: Bytes,
}

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
#[ethevent(name = "Transfer")]
pub struct TransferEvent {
    #[ethevent(indexed)]
    pub from: Address,
    #[ethevent(indexed)]
    pub to: Address,
    pub value: U256,
}

pub async fn run() -> eyre::Result<()> {
    let provider = Provider::<Ws>::connect(secret::ETH_RPC).await?;
    let provider = Arc::new(provider);

    let contract_address = constant::GPv2SETTLEMENT.parse::<Address>()?;
    let trade_filter = Filter::new().address(contract_address);

    let trade_event = TradeEvent::new::<_, Provider<Ws>>(trade_filter, Arc::clone(&provider));
    let mut trades = trade_event.subscribe().await?.take(1);
    while let Some(trade) = trades.next().await {
        let trade = trade?;
        println!("Trade: {:?}", trade);

        let transfer_filter = Filter::new()
            .event("Transfer(address,address,uint256)")
            .address(ValueOrArray::Array(vec![trade.sell_token, trade.buy_token]));

        let transfer_event =
            TransferEvent::new::<_, Provider<Ws>>(transfer_filter, Arc::clone(&provider));
        let transfers = transfer_event.query().await?;

        for transfer in transfers {
            println!("Related Transfer: {:?}", transfer);
        }
    }

    Ok(())
}
