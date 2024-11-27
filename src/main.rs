use cow_quote::run;
use cow_quote::services::aws_lambda::{is_running_in_aws_lambda, lambda_handler};
use lambda_runtime::{run as lambda_run, service_fn};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if is_running_in_aws_lambda() {
        println!("Running AWS Lambda");
        lambda_run(service_fn(lambda_handler))
            .await
            .map_err(|e| eyre::eyre!(e))?;
        Ok(())
    } else {
        println!("Running locally");
        run().await
    }
}
