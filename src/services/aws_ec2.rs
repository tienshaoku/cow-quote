use crate::helper::EnvConfig;
use crate::run;
use aws-sdk-ec2::Error;
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Serialize)]
pub struct Response {
    status: String,
    message: String,
}

pub async fn handle_request(config: EnvConfig) -> Result<Response, Error> {
    println!("handle_request() on AWS EC2");

    let duration = 15 * 60;

    match timeout(Duration::from_secs(duration), run(config)).await {
        Ok(result) => match result {
            Ok(_) => {
                let message = String::from("Function completed successfully");
                println!("{}", message);
                Ok(Response {
                    status: "success".to_string(),
                    message,
                })
            }
            Err(e) => {
                let message = format!("Function failed with error: {}", e);
                println!("{}", message);
                Ok(Response {
                    status: "error".to_string(),
                    message,
                })
            }
        },
        Err(_) => {
            let message = format!("Function timed out after {} seconds", duration);
            println!("{}", message);
            Ok(Response {
                status: "timeout".to_string(),
                message,
            })
        }
    }
}

pub fn is_running_in_aws_ec2() -> bool {
    std::env::var("EC2_INSTANCE_ID").is_ok()
}
