use super::load_env;
use base64::{engine::general_purpose, Engine as _};
use serde_json;
use std::env;
use std::time::{Duration, SystemTime};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest, WebSocketStream};

use super::cognito_auth::{
    authenticate_with_cognito, create_test_cognito_user, delete_cognito_user,
};

fn get_ws_url() -> String {
    load_env();
    env::var("WEBSOCKET_URL")
        .unwrap_or_else(|_| panic!("WEBSOCKET_URL environment variable not set."))
}

/// Creates a test user and returns their credentials and user_id
async fn setup_test_user(test_email: &str, test_password: &str) -> (String, String) {
    // Ensure environment variables are loaded before AWS SDK initialization
    load_env();

    // Create a new Cognito user
    create_test_cognito_user(test_email, test_password)
        .await
        .expect("Failed to create test Cognito user");

    // Authenticate with the new user
    let tokens = authenticate_with_cognito(test_email, test_password)
        .await
        .expect("Failed to authenticate with test user");

    // Decode the JWT to extract the user_id (sub)
    let jwt_parts: Vec<&str> = tokens.id_token.split('.').collect();
    assert_eq!(jwt_parts.len(), 3, "Invalid JWT format");
    let payload = general_purpose::URL_SAFE_NO_PAD
        .decode(jwt_parts[1])
        .expect("Invalid JWT payload");
    let claims: serde_json::Value = serde_json::from_slice(&payload).expect("Invalid claims JSON");
    let user_id = claims["sub"]
        .as_str()
        .expect("Missing sub in JWT claims")
        .to_string();

    println!("User ID: {}", user_id);
    println!("Token ID: {}", tokens.id_token);

    (user_id, tokens.id_token)
}

/// Connects to the WebSocket with the provided token
async fn connect_websocket(
    id_token: &str,
) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
    let ws_url = get_ws_url();
    println!("[connect_websocket] WebSocket URL: {}", ws_url);

    let mut request = ws_url
        .into_client_request()
        .expect("Failed to create WebSocket request");

    println!("[connect_websocket] Adding Authorization header");
    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", id_token)
            .parse()
            .expect("Failed to parse Authorization header"),
    );

    println!("[connect_websocket] Attempting to connect...");
    let (ws_stream, response) = connect_async(request)
        .await
        .expect("Failed to connect to WebSocket");

    println!("[connect_websocket] Connected to WebSocket successfully");
    println!("[connect_websocket] Response status: {}", response.status());
    ws_stream
}

/// Sends a message to the WebSocket and validates the response
async fn send_and_validate_response(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    message: serde_json::Value,
    expected_status: &str,
    expected_message: &str,
) {
    use futures_util::{SinkExt, StreamExt};

    let action = message["action"].as_str().unwrap_or("unknown");

    println!("[send_and_validate_response] Action: {}", action);
    println!("[send_and_validate_response] Message payload: {}", message);

    // Send message
    println!("[send_and_validate_response] Sending message...");
    ws_stream
        .send(Message::Text(message.to_string()))
        .await
        .unwrap_or_else(|e| panic!("Failed to send {} message: {:?}", action, e));

    println!(
        "[send_and_validate_response] Sent {} message successfully",
        action
    );

    // Receive response
    println!("[send_and_validate_response] Waiting for response...");
    let response = ws_stream
        .next()
        .await
        .unwrap_or_else(|| panic!("No response received for {} (connection closed)", action))
        .unwrap_or_else(|e| panic!("Failed to receive {} response: {:?}", action, e));

    println!(
        "[send_and_validate_response] Received {} response: {:?}",
        action, response
    );

    // Verify the response
    if let Message::Text(text) = response {
        println!("[send_and_validate_response] Response text: {}", text);
        let response_data: serde_json::Value =
            serde_json::from_str(&text).expect("Invalid JSON response");

        println!(
            "[send_and_validate_response] Parsed response: {}",
            response_data
        );
        println!(
            "[send_and_validate_response] Expected status: {}, Actual status: {:?}",
            expected_status,
            response_data["status"].as_str()
        );
        println!(
            "[send_and_validate_response] Expected message: {}, Actual message: {:?}",
            expected_message,
            response_data["message"].as_str()
        );

        assert_eq!(
            response_data["status"].as_str(),
            Some(expected_status),
            "Expected {} status for {}. Full response: {}",
            expected_status,
            action,
            response_data
        );
        assert_eq!(
            response_data["message"].as_str(),
            Some(expected_message),
            "Expected '{}' message. Full response: {}",
            expected_message,
            response_data
        );
        println!(
            "[send_and_validate_response] {} successful: {}",
            action, text
        );
    } else {
        panic!(
            "Expected text message for {} response, got: {:?}",
            action, response
        );
    }
}

#[tokio::test]
async fn test_websocket_join_leave_queue_flow() {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_email = format!("test-websocket-queue-{timestamp}@example.com");
    let test_password = "TempPassword123!";

    println!("=== Starting test_websocket_join_leave_queue_flow ===");
    println!("Test email: {}", test_email);
    println!("Timestamp: {}", timestamp);

    // Wrap test in timeout to prevent hanging indefinitely
    let result = timeout(Duration::from_secs(30), async {
        println!("\n--- Step 1: Setting up test user ---");
        // Setup test user and authenticate
        let (_user_id, id_token) = setup_test_user(&test_email, &test_password).await;
        println!("Test user setup complete");

        println!("\n--- Step 2: Connecting to WebSocket ---");
        // Connect to WebSocket
        let mut ws_stream = connect_websocket(&id_token).await;
        println!("WebSocket connection established");

        println!("\n--- Step 3: Joining queue ---");
        // Join queue
        let join_queue_msg = serde_json::json!({
            "action": "join_queue",
            "time_control": "blitz",
            "min_rating": 1000,
            "max_rating": 2000
        });
        println!("Sending join_queue message: {}", join_queue_msg);
        send_and_validate_response(&mut ws_stream, join_queue_msg, "success", "Joined queue").await;
        println!("Successfully joined queue");

        println!("\n--- Step 4: Leaving queue ---");
        // Leave queue
        let leave_queue_msg = serde_json::json!({
            "action": "leave_queue",
            "time_control": "blitz"
        });
        println!("Sending leave_queue message: {}", leave_queue_msg);
        send_and_validate_response(&mut ws_stream, leave_queue_msg, "success", "Left queue").await;
        println!("Successfully left queue");

        println!("\n--- Step 5: Disconnecting ---");
        // Disconnect
        ws_stream
            .close(None)
            .await
            .expect("Failed to close WebSocket");

        println!("WebSocket closed successfully");
        println!("=== Test completed successfully ===\n");
    })
    .await;

    // Always attempt to clean up, regardless of test success/failure
    println!("\n--- Cleanup: Deleting test user ---");
    match delete_cognito_user(&test_email).await {
        Ok(_) => println!("Test user deleted successfully"),
        Err(e) => println!("Warning: Failed to delete test user: {:?}", e),
    }

    // Handle timeout result
    match result {
        Ok(_) => println!("Test completed successfully within timeout"),
        Err(_) => panic!("Test timed out after 30 seconds"),
    }
}
