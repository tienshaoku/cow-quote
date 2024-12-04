use cow_quote::helper::EnvConfig;
use cow_quote::run;
use cow_quote::services::aws_ec2::{handle_request, is_running_in_aws_ec2};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = EnvConfig::new();

    if is_running_in_aws_ec2() {
        println!("Running on AWS EC2");
        handle_request(config).await.map_err(|e| eyre::eyre!(e))?;
        Ok(())
    } else {
        println!("Running locally");
        run(config).await
    }
}
