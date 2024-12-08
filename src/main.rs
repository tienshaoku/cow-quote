use axum::{http::StatusCode, routing::post, Json, Router};
use cow_quote::handle_start_service;
use cow_quote::services::aws_ec2::is_running_in_aws_ec2;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if is_running_in_aws_ec2() {
        println!("Running on AWS EC2");
    } else {
        println!("Running locally");
    }

    let api_router = Router::new()
        .route("/start", post(start_service))
        // Add CORS middleware
        .layer(CorsLayer::permissive());

    // Run the API server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("API server running on http://0.0.0.0:3000");
    axum::serve(listener, api_router).await?;
    Ok(())
}

async fn start_service() -> Result<Json<String>, StatusCode> {
    match handle_start_service().await {
        Ok(message) => Ok(Json(message)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
