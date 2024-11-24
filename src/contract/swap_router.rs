use ethers::prelude::abigen;

pub use SwapRouter;

abigen!(SwapRouter, "./src/abi/SwapRouter.json");
