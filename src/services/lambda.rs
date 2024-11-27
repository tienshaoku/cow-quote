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
                println!("Function completed successfully");
                Ok(Response {
                    status: "success".to_string(),
                    message: "Completed successfully".to_string(),
                })
            }
            Err(e) => {
                println!("Function failed with error: {}", e);
                Ok(Response {
                    status: "error".to_string(),
                    message: format!("Error during execution: {}", e),
                })
            }
        },
        Err(_) => {
            println!("Function timed out");
            Ok(Response {
                status: "timeout".to_string(),
                message: format!("Function timed out after {} seconds", duration),
            })
        }
    }
}
