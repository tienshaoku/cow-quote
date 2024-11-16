use ethers::prelude::abigen;

pub use IERC20;
abigen!(
    IERC20,
    r#"[
    function decimals() public view virtual returns (uint8)
    ]"#
);
