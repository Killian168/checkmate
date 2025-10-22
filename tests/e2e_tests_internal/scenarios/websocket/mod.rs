use reqwest::StatusCode;
use std::time::Duration;

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
    for i in 0..3 {
        let ws_result = client.establish_websocket_connection(user_id).await;
        assert!(
            ws_result.is_ok(),
            "WebSocket connection {} should succeed",
            i + 1
        );
    }

    // Verify we have the expected number of connections
    assert_eq!(3, 3, "Should have established 3 connections");

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
    client.close_websocket_connections().await?;

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

/// Test game matching notifications via WebSocket
pub async fn test_game_matching_notifications() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // Wait 30 seconds to allow any ongoing processing from previous tests to complete
    println!("Waiting 30 seconds to allow previous test processing to complete...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Starting game matching notifications test...");

    // Create two players
    let mut player1_client = TestClient::from_config(&config);
    let mut player2_client = TestClient::from_config(&config);

    let player1_user = TestUser::new("Player1", "Match");
    let player2_user = TestUser::new("Player2", "Match");

    // Register and login both players

    let register_status = player1_client
        .register_user(
            &player1_user.email,
            &player1_user.password,
            &player1_user.first_name,
            &player1_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    let register_status = player2_client
        .register_user(
            &player2_user.email,
            &player2_user.password,
            &player2_user.first_name,
            &player2_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    let token1 = player1_client
        .login(&player1_user.email, &player1_user.password)
        .await?;
    assert!(!token1.is_empty());

    let token2 = player2_client
        .login(&player2_user.email, &player2_user.password)
        .await?;
    assert!(!token2.is_empty());

    // Get user info to extract user IDs

    let player1_info = player1_client.get_user_info().await?;
    let player1_id = player1_info["id"]
        .as_str()
        .ok_or("Player 1 ID not found in response")?;

    let player2_info = player2_client.get_user_info().await?;
    let player2_id = player2_info["id"]
        .as_str()
        .ok_or("Player 2 ID not found in response")?;

    // Establish WebSocket connections for both players

    let ws_result1 = player1_client
        .establish_websocket_connection(player1_id)
        .await;
    assert!(
        ws_result1.is_ok(),
        "Player 1 WebSocket connection should succeed"
    );

    let ws_result2 = player2_client
        .establish_websocket_connection(player2_id)
        .await;
    assert!(
        ws_result2.is_ok(),
        "Player 2 WebSocket connection should succeed"
    );

    // Both players join the queue

    let join_status1 = player1_client.join_queue("rapid").await?;
    assert_eq!(join_status1, StatusCode::OK);

    let join_status2 = player2_client.join_queue("rapid").await?;
    assert_eq!(join_status2, StatusCode::OK);

    // Wait for game matching notifications
    let notification_timeout = Duration::from_secs(30); // Allow time for queue processing

    let player1_notification = player1_client
        .wait_for_websocket_message("game_matched", notification_timeout)
        .await?;

    let player2_notification = player2_client
        .wait_for_websocket_message("game_matched", notification_timeout)
        .await?;

    // Verify notification content for player 1
    assert_eq!(
        player1_notification["action"].as_str(),
        Some("game_matched"),
        "Player 1 should receive game_matched action"
    );
    assert!(
        player1_notification["session_id"].is_string(),
        "Player 1 notification should contain session_id"
    );
    assert_eq!(
        player1_notification["opponent_id"].as_str(),
        Some(player2_id),
        "Player 1 should see player 2 as opponent"
    );
    assert!(
        player1_notification["message"].is_string(),
        "Player 1 notification should contain message"
    );

    // Verify notification content for player 2
    assert_eq!(
        player2_notification["action"].as_str(),
        Some("game_matched"),
        "Player 2 should receive game_matched action"
    );
    assert!(
        player2_notification["session_id"].is_string(),
        "Player 2 notification should contain session_id"
    );
    assert_eq!(
        player2_notification["opponent_id"].as_str(),
        Some(player1_id),
        "Player 2 should see player 1 as opponent"
    );
    assert!(
        player2_notification["message"].is_string(),
        "Player 2 notification should contain message"
    );

    // Verify both players received the same session ID
    let session_id1 = player1_notification["session_id"]
        .as_str()
        .expect("Player 1 session_id should be string");
    let session_id2 = player2_notification["session_id"]
        .as_str()
        .expect("Player 2 session_id should be string");
    assert_eq!(
        session_id1, session_id2,
        "Both players should receive the same session ID"
    );

    // Clean up WebSocket connections
    player1_client.close_websocket_connections().await?;
    player2_client.close_websocket_connections().await?;

    // Clean up accounts
    let delete_status1 = player1_client.delete_account().await?;
    assert_eq!(delete_status1, StatusCode::NO_CONTENT);

    let delete_status2 = player2_client.delete_account().await?;
    assert_eq!(delete_status2, StatusCode::NO_CONTENT);

    timer.assert_within_timeout(config.timeout);
    Ok(())
}
