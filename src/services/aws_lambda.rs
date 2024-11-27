use crate::run;
use lambda_runtime::{Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Deserialize, Debug)]
pub struct Request {
    run_duration: Option<u64>,
}

#[derive(Serialize)]
pub struct Response {
    status: String,
    message: String,
}

pub async fn lambda_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    println!("Received payload: {:?}", event.payload);

    let duration = event.payload.run_duration.unwrap_or(14 * 60);

    match timeout(Duration::from_secs(duration), run()).await {
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

pub fn is_running_in_aws_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}
