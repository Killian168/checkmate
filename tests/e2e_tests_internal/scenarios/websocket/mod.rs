use reqwest::StatusCode;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::e2e_tests_internal::client::TestClient;
use crate::e2e_tests_internal::config::E2EConfig;
use crate::e2e_tests_internal::utils::{TestResult, TestTimer, TestUser};

/// Test basic WebSocket connection flow
pub async fn test_websocket_connection_flow() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let mut client = TestClient::from_config(&config);

    let test_user = TestUser::new("WebSocket", "Test");

    // Register and login
    let register_status = client
        .register_user(
            &test_user.email,
            &test_user.password,
            &test_user.first_name,
            &test_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&test_user.email, &test_user.password).await?;
    assert!(!token.is_empty());

    // Get user info to extract user ID
    let user_info = client.get_user_info().await?;
    let user_id = user_info["id"]
        .as_str()
        .ok_or("User ID not found in response")?;

    // Establish WebSocket connection
    let ws_result = client.establish_websocket_connection(user_id).await;
    assert!(ws_result.is_ok(), "WebSocket connection should succeed");

    // Clean up
    let delete_status = client.delete_account().await?;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test multiple WebSocket connections from the same user
pub async fn test_websocket_multiple_connections() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let mut client = TestClient::from_config(&config);

    let test_user = TestUser::new("MultiWS", "Test");

    // Register and login
    let register_status = client
        .register_user(
            &test_user.email,
            &test_user.password,
            &test_user.first_name,
            &test_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&test_user.email, &test_user.password).await?;
    assert!(!token.is_empty());

    // Get user info
    let user_info = client.get_user_info().await?;
    let user_id = user_info["id"]
        .as_str()
        .ok_or("User ID not found in response")?;

    // Establish multiple WebSocket connections
    let mut connections: Vec<WebSocketStream<MaybeTlsStream<TcpStream>>> = Vec::new();
    for i in 0..3 {
        let ws_result = client.establish_websocket_connection(user_id).await;
        assert!(
            ws_result.is_ok(),
            "WebSocket connection {} should succeed",
            i + 1
        );
        connections.push(ws_result?);
    }

    // Verify we have the expected number of connections
    assert_eq!(
        connections.len(),
        3,
        "Should have established 3 connections"
    );

    // Clean up
    let delete_status = client.delete_account().await?;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test WebSocket reconnection scenarios
pub async fn test_websocket_reconnection() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let mut client = TestClient::from_config(&config);

    let test_user = TestUser::new("Reconnect", "Test");

    // Register and login
    let register_status = client
        .register_user(
            &test_user.email,
            &test_user.password,
            &test_user.first_name,
            &test_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&test_user.email, &test_user.password).await?;
    assert!(!token.is_empty());

    // Get user info
    let user_info = client.get_user_info().await?;
    let user_id = user_info["id"]
        .as_str()
        .ok_or("User ID not found in response")?;

    // First connection
    let ws_result1 = client.establish_websocket_connection(user_id).await;
    assert!(
        ws_result1.is_ok(),
        "First WebSocket connection should succeed"
    );

    // Simulate reconnection - close first connection and create new one
    drop(ws_result1?);

    // Second connection (reconnection)
    let ws_result2 = client.establish_websocket_connection(user_id).await;
    assert!(
        ws_result2.is_ok(),
        "Second WebSocket connection (reconnection) should succeed"
    );

    // Clean up
    let delete_status = client.delete_account().await?;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    timer.assert_within_timeout(config.timeout);
    Ok(())
}
