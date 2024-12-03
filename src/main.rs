use cow_quote::run;
use cow_quote::services::aws_lambda::{handle_request, is_running_in_aws_ec2};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if is_running_in_aws_ec2() {
        println!("Running on AWS EC2");
        handle_request().await.map_err(|e| eyre::eyre!(e))?;
        Ok(())
    } else {
        println!("Running locally");
        run().await
    }
}
