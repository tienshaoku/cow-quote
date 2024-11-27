use cow_quote::run;
use cow_quote::services::lambda::lambda_handler;
use lambda_runtime::{run as lambda_run, service_fn};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Check if running in AWS Lambda
    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        println!("Running AWS Lambda");
        // Lambda execution
        lambda_run(service_fn(lambda_handler))
            .await
            .map_err(|e| eyre::eyre!(e))?;
        Ok(())
    } else {
        println!("Running locally");
        // Local execution
        run().await
    }
}
