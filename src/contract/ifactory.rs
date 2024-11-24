use ethers::prelude::abigen;

pub use IFactory;

abigen!(
    IFactory,
    r#"[
    function getPool(address, address, uint24) external view returns (address)
    ]"#
);
