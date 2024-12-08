use cow_quote::services::aws_ec2::is_running_in_aws_ec2;
use cow_quote::{run, run_with_timeout};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if is_running_in_aws_ec2() {
        println!("Running on AWS EC2");
        let message = run_with_timeout().await?;
        println!("{}", message);
        Ok(())
    } else {
        println!("Running locally");
        run().await
    }
}
