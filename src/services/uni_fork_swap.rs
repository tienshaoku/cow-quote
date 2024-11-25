use crate::contract::{
    ierc20::IERC20,
    ifactory::IFactory,
    swap_router::{ExactInputSingleParams, SwapRouter},
};
use crate::{constant, secret};
use ethers::{
    core::utils::{parse_ether, Anvil},
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub async fn uni_swap_buy(
    block_number: u64,
    owner: &str,
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
) -> eyre::Result<String> {
    let forked_block_number = block_number - 1;
    let anvil = Anvil::new()
        .fork(secret::HTTP_ETH_RPC)
        .chain_id(1_u64)
        .fork_block_number(forked_block_number)
        .spawn();

    let http_provider =
        Provider::<Http>::try_from(anvil.endpoint()).expect("Failed to create HTTP provider");
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
    let sell_token = sell_token.parse::<Address>()?;
    let buy_token = buy_token.parse::<Address>()?;
    let signer = (*http_provider).clone().with_sender(owner);
    let erc20 = IERC20::new(sell_token, signer.clone().into());

    let balance = erc20.balance_of(owner).call().await?;
    let sell_amount = U256::from_dec_str(sell_amount)?;
    if balance < sell_amount {
        return Err(eyre::eyre!(
            "Insufficient balance: {} < {}",
            balance,
            sell_amount
        ));
    }

    let swap_router = constant::UNISWAP_V3_ROUTER.parse::<Address>()?;
    let approval_tx = erc20.approve(swap_router, U256::max_value());
    let _receipt = approval_tx.send().await?.await?;

    let approval = erc20.allowance(owner, swap_router).call().await?;
    if approval < sell_amount {
        return Err(eyre::eyre!("Max approval failed"));
    }

    let swap_router = SwapRouter::new(swap_router, signer.clone().into());
    let factory = IFactory::new(
        constant::UNISWAP_V3_FACTORY.parse::<Address>()?,
        signer.clone().into(),
    );

    let mut max_amount_out = U256::default();
    let fee_tiers = [100, 500, 3000, 10000];

    for fee in fee_tiers {
        match factory.get_pool(sell_token, buy_token, fee).call().await {
            Ok(pool_address) if pool_address != Address::zero() => (),
            _ => continue,
        };

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

        let time_out = Duration::from_secs(8);
        // Try to execute the swap
        let pending_tx = match timeout(time_out, tx.send()).await {
            Ok(Ok(tx)) => tx,
            Ok(Err(e)) => {
                println!("Failed to send transaction: {}", e);
                continue;
            }
            Err(_) => {
                println!("Transaction submission timed out");
                continue;
            }
        };

        if timeout(time_out, pending_tx).await.is_err() {
            println!("Transaction confirmation timed out");
            continue;
        }

        let balance_after = erc20.balance_of(owner).call().await?;
        let amount_out = balance_after - balance_before;
        if amount_out > max_amount_out {
            max_amount_out = amount_out;
        }
    }
    drop(anvil);

    if max_amount_out == U256::default() {
        Err(eyre::eyre!("No successful swap at all"))
    } else {
        Ok(max_amount_out.to_string())
    }
}
