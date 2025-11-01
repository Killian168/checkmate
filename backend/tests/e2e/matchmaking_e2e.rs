use super::load_env;
use base64::{engine::general_purpose, Engine as _};
use futures_util::{SinkExt, StreamExt};
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

    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", id_token)
            .parse()
            .expect("Failed to parse Authorization header"),
    );

    let (ws_stream, response) = connect_async(request)
        .await
        .expect("Failed to connect to WebSocket");

    println!("[connect_websocket] Connected to WebSocket successfully");
    println!("[connect_websocket] Response status: {}", response.status());
    ws_stream
}

/// Sends a message to the WebSocket
async fn send_message(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    message: serde_json::Value,
) {
    println!("[send_message] Sending: {}", message);
    ws_stream
        .send(Message::Text(message.to_string()))
        .await
        .expect("Failed to send message");
}

/// Receives and validates a response from the WebSocket
async fn receive_message(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
) -> serde_json::Value {
    let response = ws_stream
        .next()
        .await
        .expect("No response received")
        .expect("Failed to receive response");

    if let Message::Text(text) = response {
        println!("[receive_message] Received: {}", text);
        serde_json::from_str(&text).expect("Invalid JSON response")
    } else {
        panic!("Expected text message, got: {:?}", response);
    }
}

#[tokio::test]
async fn test_matchmaking_two_users() {
    // Add a small delay to ensure
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_email1 = format!("test-matchmaking-1-{timestamp}@example.com");
    let test_email2 = format!("test-matchmaking-2-{timestamp}@example.com");
    let test_password = "TempPassword123!";

    println!("=== Starting test_matchmaking_two_users ===");
    println!("Test email 1: {}", test_email1);
    println!("Test email 2: {}", test_email2);
    println!("Timestamp: {}", timestamp);

    let result = timeout(Duration::from_secs(90), async {
        println!("\n--- Step 1: Setting up two test users ---");
        let (user_id1, id_token1) = setup_test_user(&test_email1, &test_password).await;
        let (user_id2, id_token2) = setup_test_user(&test_email2, &test_password).await;
        println!("Test users created: {} and {}", user_id1, user_id2);

        println!("\n--- Step 2: Connecting both users to WebSocket ---");
        let mut ws_stream1 = connect_websocket(&id_token1).await;
        let mut ws_stream2 = connect_websocket(&id_token2).await;
        println!("Both WebSocket connections established");

        println!("\n--- Step 3: Both users join the queue ---");
        let join_queue_msg1 = serde_json::json!({
            "action": "join_queue",
            "time_control": "blitz",
            "min_rating": 1000,
            "max_rating": 2000
        });
        send_message(&mut ws_stream1, join_queue_msg1).await;
        let response1 = receive_message(&mut ws_stream1).await;
        assert_eq!(response1["status"].as_str(), Some("success"));
        println!("User 1 joined queue successfully");

        let join_queue_msg2 = serde_json::json!({
            "action": "join_queue",
            "time_control": "blitz",
            "min_rating": 1000,
            "max_rating": 2000
        });
        send_message(&mut ws_stream2, join_queue_msg2).await;
        let response2 = receive_message(&mut ws_stream2).await;
        assert_eq!(response2["status"].as_str(), Some("success"));
        println!("User 2 joined queue successfully");

        println!("\n--- Step 4: Wait for matchmaker to run and match users (up to 70 seconds) ---");
        // Matchmaker runs every minute, so we need to wait for it to run
        // We'll wait for up to 70 seconds for the match notification

        let match_result1 = timeout(Duration::from_secs(70), async {
            loop {
                if let Some(Ok(Message::Text(text))) = ws_stream1.next().await {
                    println!("[User 1] Received message: {}", text);
                    let msg: serde_json::Value = serde_json::from_str(&text).expect("Invalid JSON");
                    if msg["action"].as_str() == Some("game_matched") {
                        return msg;
                    }
                }
            }
        })
        .await
        .expect("Timed out waiting for match notification for user 1");

        let match_result2 = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(Ok(Message::Text(text))) = ws_stream2.next().await {
                    println!("[User 2] Received message: {}", text);
                    let msg: serde_json::Value = serde_json::from_str(&text).expect("Invalid JSON");
                    if msg["action"].as_str() == Some("game_matched") {
                        return msg;
                    }
                }
            }
        })
        .await
        .expect("Timed out waiting for match notification for user 2");

        println!("\n--- Step 5: Verify match notifications ---");
        assert_eq!(match_result1["action"].as_str(), Some("game_matched"));
        assert_eq!(match_result2["action"].as_str(), Some("game_matched"));

        let game_id1 = match_result1["game_id"].as_str().expect("Missing game_id");
        let game_id2 = match_result2["game_id"].as_str().expect("Missing game_id");

        let opponent_id1 = match_result1["opponent_id"]
            .as_str()
            .expect("Missing opponent_id");
        let opponent_id2 = match_result2["opponent_id"]
            .as_str()
            .expect("Missing opponent_id");

        let color1 = match_result1["color"].as_str().expect("Missing color");
        let color2 = match_result2["color"].as_str().expect("Missing color");
        assert!(
            (color1 == "white" && color2 == "black") || (color1 == "black" && color2 == "white"),
            "Colors should be white and black"
        );

        println!("\n--- Match successful! ---");
        println!("Game ID: {}", game_id1);
        println!("User 1 ({}) is {}", user_id1, color1);
        println!("User 2 ({}) is {}", user_id2, color2);

        println!("\n--- Step 6: Disconnecting ---");
        ws_stream1
            .close(None)
            .await
            .expect("Failed to close WebSocket 1");
        ws_stream2
            .close(None)
            .await
            .expect("Failed to close WebSocket 2");

        println!("=== Test completed successfully ===\n");
    })
    .await;

    // Always attempt to clean up
    println!("\n--- Cleanup: Deleting test users ---");
    match delete_cognito_user(&test_email1).await {
        Ok(_) => println!("Test user 1 deleted successfully"),
        Err(e) => println!("Warning: Failed to delete test user 1: {:?}", e),
    }
    match delete_cognito_user(&test_email2).await {
        Ok(_) => println!("Test user 2 deleted successfully"),
        Err(e) => println!("Warning: Failed to delete test user 2: {:?}", e),
    }

    // Handle timeout result
    match result {
        Ok(_) => println!("Test completed successfully within timeout"),
        Err(_) => panic!("Test timed out after 90 seconds"),
    }
}
