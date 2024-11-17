use ethers::prelude::*;

pub use SwapRouter;
abigen!(SwapRouter, "src/abi/SwapRouter.json");

impl<M: Middleware> SwapRouter<M> {
    pub async fn simulate_exact_input_single(
        &self,
        params: swap_router::ExactInputSingleParams,
        block_number: U64,
    ) -> Result<U256, ContractError<M>> {
        self.exact_input_single(params)
            .block(BlockNumber::Number(block_number - 1))
            .call()
            .await
    }

    // pub async fn simulate_exact_output_single(
    //     &self,
    //     params: swap_router::ExactOutputSingleParams,
    // ) -> Result<U256, ContractError<M>> {
    //     self.exact_output_single(params).call().await
    // }
}
