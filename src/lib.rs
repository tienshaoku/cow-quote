mod constant;
mod format;
mod ierc20;
mod order;
#[path = "secret.rs"]
mod secret;
pub mod services;

use ethers::{
    contract::EthEvent,
    core::utils::Anvil,
    middleware::Middleware,
    providers::{Http, Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use futures::StreamExt;
use ierc20::IERC20;
use order::Order;
use serde::{Deserialize, Serialize};
use services::{
    cow_get_order_api::{cowswap_get_order, CowGetResponse},
    cow_post_quote_api::cowswap_quote_buy,
    zerox_get_quote_api::zerox_get_quote,
};
use std::sync::Arc;

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

const TRIAL_COUNT: usize = 4;
const SWAP_ROUTER: &str = "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45";

pub async fn run() -> eyre::Result<()> {
    let ws_provider = Provider::<Ws>::connect(secret::ETH_RPC).await?;
    let ws_provider = Arc::new(ws_provider);

    let settlement_contract = constant::GPv2SETTLEMENT.parse::<Address>()?;
    let trade_filter = Filter::new().address(settlement_contract);

    let trade_event = TradeEvent::new::<_, Provider<Ws>>(trade_filter, Arc::clone(&ws_provider));
    let mut stream = trade_event.stream().await?.with_meta().take(TRIAL_COUNT);
    while let Some(Ok((trade, meta))) = stream.next().await {
        // println!("Trade: {:#?}\n", trade);
        let order_uid = trade.order_uid;

        let cow_api_response: CowGetResponse = cowswap_get_order(&order_uid.to_string()).await?;
        // println!("Order Response: {:#?}\n", cow_api_response);

        // TODO: see if can throw this into a thread
        // 0x has only sell orders
        if cow_api_response.is_sell() && cow_api_response.sell() == cow_api_response.executed_sell()
        {
            let block_number = meta.block_number.as_u64();
            let timestamp = if let Some(block) = &ws_provider.get_block(block_number).await? {
                block.timestamp
            } else {
                continue;
            };

            let owner = cow_api_response.owner();
            let sell_token = cow_api_response.sell_token();
            let sell_amount = cow_api_response.sell();
            let buy_token = cow_api_response.buy_token();

            let mut order = Order::from_cow_api_response(
                Arc::clone(&ws_provider),
                order_uid.to_string(),
                block_number,
                timestamp.as_u64(),
                &cow_api_response,
            )
            .await?;

            let zerox_response =
                zerox_get_quote("1", sell_token, buy_token, sell_amount, owner).await?;
            // println!("0x Response: {:#?}\n", zerox_response);

            // TODO: include gas cost on zerox but the calculation is v complicated
            order.update_zerox_comparison(zerox_response);

            let cows_own_quote_buy =
                cowswap_quote_buy(owner, sell_token, buy_token, sell_amount).await?;

            order.update_cows_own_quote_comparison(&cows_own_quote_buy);
            println!("Order: {:#?}\n", order);

            let anvil = Anvil::new()
                .fork(secret::ETH_RPC)
                .fork_block_number(block_number)
                .spawn();

            let http_provider = Provider::<Http>::try_from(anvil.endpoint())?;
            let http_provider = Arc::new(http_provider);

            assert_eq!(
                http_provider.get_block_number().await?.as_u64(),
                block_number
            );

            http_provider
                .request::<_, ()>("anvil_impersonateAccount", vec![owner.to_string()])
                .await?;

            let owner = owner.parse::<Address>()?;
            let signer = (*http_provider).clone().with_sender(owner);

            let erc20 = IERC20::new(sell_token.parse::<Address>()?, signer.into());

            let swap_router = SWAP_ROUTER.parse::<Address>()?;
            let approval = erc20.allowance(owner, swap_router).await?;
            println!("Approval before: {:?}", approval);

            erc20
                .approve(swap_router, U256::from_dec_str(sell_amount)?)
                .send()
                .await?;

            let approval = erc20.allowance(owner, swap_router).await?;
            println!("Approval after: {:?}", approval);

            drop(anvil);
        }
    }

    Ok(())
}
