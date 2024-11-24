mod constant;
mod contract;
mod format;
mod order;
#[path = "secret.rs"]
mod secret;
pub mod services;

use contract::{
    ierc20::IERC20,
    ifactory::IFactory,
    swap_router::{ExactInputSingleParams, SwapRouter},
};
use ethers::{
    contract::EthEvent,
    core::utils::{parse_ether, Anvil},
    middleware::Middleware,
    providers::{Http, Provider, Ws},
    types::{Address, Bytes, Filter, U256},
};
use futures::StreamExt;
use order::Order;
use serde::{Deserialize, Serialize};
use services::{
    cow_get_order_api::{cowswap_get_order, CowGetResponse},
    cow_post_quote_api::cowswap_quote_buy,
    zerox_get_quote_api::zerox_get_quote,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

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

pub async fn run() -> eyre::Result<()> {
    let ws_provider = Provider::<Ws>::connect(secret::WSS_ETH_RPC).await?;
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
            println!("Block number: {:?}", block_number);
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

            let forked_block_number = block_number - 1;
            let anvil = Anvil::new()
                .fork(secret::HTTP_ETH_RPC)
                .chain_id(1_u64)
                .fork_block_number(forked_block_number)
                .spawn();

            let http_provider = Provider::<Http>::try_from(anvil.endpoint())
                .expect("Failed to create HTTP provider");
            let http_provider = Arc::new(http_provider);

            http_provider
                .request::<_, ()>("anvil_impersonateAccount", vec![owner.to_string()])
                .await
                .expect("Failed to impersonate account");

            http_provider
                .request::<_, ()>(
                    "anvil_setBalance",
                    vec![owner.to_string(), parse_ether(100).unwrap().to_string()],
                )
                .await
                .expect("Failed to set balance");

            let owner = owner.parse::<Address>()?;
            let signer = (*http_provider).clone().with_sender(owner);

            let sell_token = sell_token.parse::<Address>()?;
            let buy_token = buy_token.parse::<Address>()?;

            let erc20 = IERC20::new(sell_token, signer.clone().into());

            let balance = erc20.balance_of(owner).call().await?;
            let sell_amount = U256::from_dec_str(sell_amount)?;
            if balance < sell_amount {
                println!("Insufficient balance: {} < {}", balance, sell_amount);
                continue;
            }

            let swap_router = constant::UNISWAP_V3_ROUTER.parse::<Address>()?;

            let approval_tx = erc20.approve(swap_router, U256::max_value());
            let _receipt = approval_tx.send().await?.await?;

            let approval = erc20.allowance(owner, swap_router).call().await?;
            if approval < sell_amount {
                println!("Approval failed: {} < {}", approval, sell_amount);
                continue;
            }

            let swap_router = SwapRouter::new(swap_router, signer.clone().into());
            let factory = IFactory::new(
                constant::UNISWAP_V3_FACTORY.parse::<Address>()?,
                signer.clone().into(),
            );

            let fee_tier = [100, 500, 3000, 10000];

            let mut max_amount_out = U256::default();
            for fee in fee_tier {
                let pool_address = factory.get_pool(sell_token, buy_token, fee).call().await?;
                if pool_address != Address::zero() {
                    println!("Pool address for fee {}: {:?}", fee, pool_address);
                } else {
                    println!("No pool exists for fee tier {}", fee);
                    continue;
                }

                let erc20 = IERC20::new(buy_token, signer.clone().into());
                let balance_before = erc20.balance_of(owner).call().await?;

                let tx = swap_router.exact_input_single(ExactInputSingleParams {
                    token_in: sell_token,
                    token_out: buy_token,
                    fee,
                    amount_in: sell_amount,
                    amount_out_minimum: U256::from(0),
                    recipient: owner,
                    sqrt_price_limit_x96: U256::from(0),
                });

                let pending_tx = match timeout(Duration::from_secs(30), tx.send()).await {
                    Ok(Ok(tx)) => tx,
                    Ok(Err(e)) => {
                        println!("Failed to send transaction: {:?}", e);
                        continue;
                    }
                    Err(_) => {
                        println!("Transaction submission timed out");
                        continue;
                    }
                };

                if timeout(Duration::from_secs(30), pending_tx).await.is_err() {
                    println!("Transaction confirmation timed out");
                    continue;
                }

                let balance_after = erc20.balance_of(owner).call().await?;
                let amount_out = balance_after - balance_before;
                if amount_out > max_amount_out {
                    max_amount_out = amount_out;
                }
            }
            println!("Max amount out: {:?}\n", max_amount_out);
            order.update_univ3_swap_comparison(&max_amount_out.to_string());

            println!("Order: {:#?}\n", order);

            drop(anvil);
        }
    }

    Ok(())
}
