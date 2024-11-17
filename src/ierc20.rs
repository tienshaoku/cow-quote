use ethers::prelude::abigen;

pub use IERC20;

use ethers::{
    providers::{Provider, Ws},
    types::Address,
};
use std::sync::Arc;

abigen!(
    IERC20,
    r#"[
    function decimals() public view virtual returns (uint8)
    function approve(address spender, uint256 amount) public virtual returns (bool)
    function balanceOf(address owner) public view virtual returns (uint256)
    function allowance(address owner, address spender) public view virtual returns (uint256)
    ]"#
);

pub async fn get_token_decimals(
    provider: Arc<Provider<Ws>>,
    token: Address,
    is_weth: bool,
) -> eyre::Result<u32> {
    if is_weth {
        Ok(18)
    } else {
        let erc20 = IERC20::new(token, provider);
        Ok(erc20.decimals().call().await? as u32)
    }
}
