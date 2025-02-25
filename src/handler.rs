use aws_sdk_sqs::Client;
use axum::{
    body::{to_bytes, Body},
    extract::Request,
    response::Response,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{Duration, SystemTime};

// Version string for debugging, because deployment sometimes
// seems to fail quietly
const VERSION: u8 = 8;

// Format of the message that will be added to
// the queue as a JSON string
#[derive(Serialize, Deserialize)]
struct QueueMessage {
    received: u64,
    from: String,
    body: String,
}

// Just to make types easier to read
type ErrorString = String;

/// Load SQS Queue client
async fn get_client() -> Client {
    dotenv::dotenv().ok();
    let config = aws_config::from_env()
        .endpoint_url("https://sqs.mnq.nl-ams.scaleway.com")
        .load()
        .await;
    let client = Client::new(&config);
    client
}

/// Format log message
fn log(log_type: &str, message: String) {
    println!("[{:?}|v{:?}] {:?}", log_type, VERSION, message);
}

/// Return an error response
fn build_error(message: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/plain")
        .body(Body::from(message))
        .unwrap()
}

/// Parse and format JSON message for the queue
async fn format_message(req: Request<Body>, timestamp: Duration) -> Result<String, ErrorString> {
    let (parts, body) = req.into_parts();

    // Unwrap User Agent header
    let user_agent_value = match parts.headers.get("User-Agent") {
        Some(value) => value,
        None => return Err("No user agent found".to_string()),
    };

    // Parse string
    let user_agent = match user_agent_value.to_str() {
        Ok(value) => value,
        Err(err) => return Err(format!("No valid user agent found: {:?}", err)),
    };

    // Unwrap body
    let bytes = match to_bytes(body, usize::MAX).await {
        Ok(value) => value,
        Err(err) => return Err(format!("Could not parse body: {:?}", err)),
    };

    // Parse to string
    let body = match String::from_utf8(bytes.to_vec()) {
        Ok(value) => value,
        Err(err) => return Err(format!("Could not parse body: {:?}", err)),
    };

    // Format queue message
    let message = QueueMessage {
        received: timestamp.as_secs(),
        from: user_agent.to_string(),
        body: body,
    };

    // Serialize it to a JSON string.
    let json_body = match serde_json::to_string(&message) {
        Ok(value) => value,
        Err(err) => return Err(format!("Could not format JSON: {:?}", err)),
    };

    Ok(json_body)
}

/// The function that gets called by Scaleway Serverless
pub async fn handle(req: Request<Body>) -> Response<Body> {
    log("notice", "Processing new message".to_string());

    // Get received timestamp
    // An error should technically not return a 400
    let timestamp: Duration = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(timestamp) => timestamp,
        Err(err) => {
            log("error", format!("Error: {:?}", err));
            return build_error("Could not get timestamp".to_string());
        }
    };

    // Fetch Queue url
    // An error should technically not return a 400
    let queue_url: String = match env::var("QUEUE_URL") {
        Ok(url) => url,
        Err(err) => {
            log("error", format!("Error: {:?}", err));
            return build_error("Could not parse queue url".to_string());
        }
    };

    // Format message
    let json_body = match format_message(req, timestamp).await {
        Ok(body) => body,
        Err(err) => {
            log("error", format!("Error: {:?}", err));
            return build_error("Could not format queue message".to_string());
        }
    };

    // Body > 256KB: Error
    if json_body.len() > 256000 {
        log("error", "Error: message body > 256 kbytes".to_string());
        return build_error("Body too long".to_string());
    }

    // Send message to queue
    let result = get_client()
        .await
        .send_message()
        .message_body(json_body)
        .queue_url(queue_url)
        .send()
        .await;

    if result.is_ok() {
        log("notice", "Message added to queue.".to_string());
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(Body::from("OK"))
            .unwrap()
    } else {
        log("error", format!("Error: {:?}", result.err()));
        build_error("Could not connect to queue".to_string())
    }
}
