use axum::{
    body::{to_bytes, Body}, extract::Request, response::Response,
};
use http::StatusCode;
use aws_sdk_sqs::Client;
use std::env;

// Version string for debugging
const VERSION: u8 = 1;

/// Load SQS Queue client
async fn get_client() -> Client {
    dotenv::dotenv().ok();
    let config = aws_config::from_env().endpoint_url("https://sqs.mnq.nl-ams.scaleway.com").load().await;
    let client = Client::new(&config);
    client
}

/// Return an error response
pub fn build_error(message: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/plain")
        .body(Body::from(message))
        .unwrap()
}

/// The function that gets called by Scaleway Serverless
pub async fn handle(req: Request<Body>) -> Response<Body> {

    println!("Version {:?}", VERSION);
    println!("{:?}", req.headers().get("User-Agent"));

    // Unwrap body
    let bytes_result = to_bytes(req.into_body(), usize::MAX).await;
    if bytes_result.is_err() {
        println!("Error: {:?}", bytes_result.err());
        return build_error("Could not parse body".to_string());
    }
    let bytes = bytes_result.unwrap();

    // Parse to string
    let body_result = String::from_utf8(bytes.to_vec());
    if body_result.is_err() {
        println!("Error: {:?}", body_result.err());
        return build_error("Could not parse body".to_string());
    }
    let body = body_result.unwrap();

    // Body > 256KB: Error

    // Fetch Queue url
    let queue_url_result = env::var("QUEUE_URL");
    if queue_url_result.is_err() {
        println!("Error: {:?}", queue_url_result.err());
        return build_error("Could not parse queue url".to_string());
    }
    let queue_url = queue_url_result.unwrap();

    // Send message to queue
    let result = get_client()
        .await
        .send_message()
        .message_body(body)
        .queue_url(queue_url)
        .send()
        .await;

    if result.is_ok() {
        Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(Body::from("OK"))
        .unwrap()
    } else {
        println!("Error: {:?}", result.err());
        build_error("Could not connect to queue".to_string())
    }
}
