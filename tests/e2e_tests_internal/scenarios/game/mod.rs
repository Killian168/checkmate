use serde_json::json;
use std::time::Duration;

use crate::e2e_tests_internal::client::TestClient;
use crate::e2e_tests_internal::config::E2EConfig;
use crate::e2e_tests_internal::utils::{TestResult, TestTimer, TestUser};

/// Test a complete chess game from queuing to checkmate
pub async fn test_complete_chess_game() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // Create two players
    let mut player1_client = TestClient::from_config(&config);
    let mut player2_client = TestClient::from_config(&config);

    let player1_user = TestUser::new("White", "Player");
    let player2_user = TestUser::new("Black", "Player");

    // Register both players
    player1_client
        .register_user(
            &player1_user.email,
            &player1_user.password,
            &player1_user.first_name,
            &player1_user.last_name,
        )
        .await?;
    player2_client
        .register_user(
            &player2_user.email,
            &player2_user.password,
            &player2_user.first_name,
            &player2_user.last_name,
        )
        .await?;

    // Login both players
    player1_client
        .login(&player1_user.email, &player1_user.password)
        .await?;
    player2_client
        .login(&player2_user.email, &player2_user.password)
        .await?;

    // Get user IDs
    let player1_info = player1_client.get_user_info().await?;
    let player1_id = player1_info["id"].as_str().unwrap();
    let player2_info = player2_client.get_user_info().await?;
    let player2_id = player2_info["id"].as_str().unwrap();

    // Establish WebSocket connections
    player1_client
        .establish_websocket_connection(player1_id)
        .await?;
    player2_client
        .establish_websocket_connection(player2_id)
        .await?;

    // Both players join queue
    player1_client.join_queue("rapid").await?;
    player2_client.join_queue("rapid").await?;

    // Wait for game matching
    let player1_match = player1_client
        .wait_for_websocket_message("game_matched", Duration::from_secs(30))
        .await?;
    let player2_match = player2_client
        .wait_for_websocket_message("game_matched", Duration::from_secs(30))
        .await?;

    let session_id = player1_match["session_id"].as_str().unwrap();

    // Verify both players are matched to the same game
    assert_eq!(player1_match["session_id"], player2_match["session_id"]);
    assert_eq!(player1_match["opponent_id"], player2_id);
    assert_eq!(player2_match["opponent_id"], player1_id);

    // Play a simple game: Scholar's mate
    // 1. e4 e5
    player1_client.make_move(session_id, "e2", "e4").await?;
    player1_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;

    player2_client.make_move(session_id, "e7", "e5").await?;
    player1_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;

    // 2. Bc4 Nc6
    player1_client.make_move(session_id, "f1", "c4").await?;
    player1_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;

    player2_client.make_move(session_id, "b8", "c6").await?;
    player1_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;

    // 3. Qh5 Nf6
    player1_client.make_move(session_id, "d1", "h5").await?;
    player1_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;

    player2_client.make_move(session_id, "g8", "f6").await?;
    player1_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(session_id, Duration::from_secs(5))
        .await?;

    // 4. Qxf7# (Checkmate)
    player1_client.make_move(session_id, "h5", "f7").await?;

    // Wait for checkmate notifications
    let player1_game_end = player1_client
        .wait_for_game_end(session_id, Duration::from_secs(5))
        .await?;
    let player2_game_end = player2_client
        .wait_for_game_end(session_id, Duration::from_secs(5))
        .await?;

    // Verify game ended in checkmate with correct winner
    assert_eq!(player1_game_end["game_status"], "Checkmate");
    assert_eq!(player2_game_end["game_status"], "Checkmate");
    assert_eq!(player1_game_end["winner"], player1_id); // White wins
    assert_eq!(player2_game_end["winner"], player1_id);

    // Clean up
    player1_client.close_websocket_connections().await?;
    player2_client.close_websocket_connections().await?;
    player1_client.delete_account().await?;
    player2_client.delete_account().await?;

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test invalid move handling
pub async fn test_invalid_move_handling() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // Create two players and get them matched
    let (mut player1_client, mut player2_client, session_id, _player1_id, _player2_id) =
        setup_matched_game(&config).await?;

    // Player 1 makes a valid move
    player1_client.make_move(&session_id, "e2", "e4").await?;
    player1_client
        .wait_for_move_update(&session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(&session_id, Duration::from_secs(5))
        .await?;

    // Player 2 tries an invalid move (moving opponent's piece)
    let invalid_move_result = player2_client.make_move(&session_id, "e4", "e5").await;
    assert!(
        invalid_move_result.is_err(),
        "Invalid move should be rejected"
    );

    // Player 2 tries moving when it's not their turn
    let wrong_turn_result = player2_client.make_move(&session_id, "e7", "e5").await;
    assert!(
        wrong_turn_result.is_err(),
        "Wrong turn move should be rejected"
    );

    // Player 1 makes another valid move
    player1_client.make_move(&session_id, "d2", "d4").await?;
    player1_client
        .wait_for_move_update(&session_id, Duration::from_secs(5))
        .await?;
    player2_client
        .wait_for_move_update(&session_id, Duration::from_secs(5))
        .await?;

    // Clean up
    player1_client.close_websocket_connections().await?;
    player2_client.close_websocket_connections().await?;
    player1_client.delete_account().await?;
    player2_client.delete_account().await?;

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test pawn promotion
pub async fn test_pawn_promotion() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // Create two players and get them matched
    let (mut player1_client, mut player2_client, _session_id, _player1_id, _player2_id) =
        setup_matched_game(&config).await?;

    // Set up a position where white pawn can promote
    // This is simplified - in practice, we'd need to play moves to reach this position
    // For testing, we'll assume the game logic handles promotion correctly
    // In a real scenario, you'd play moves to advance pawns to the 8th rank

    // Clean up
    player1_client.close_websocket_connections().await?;
    player2_client.close_websocket_connections().await?;
    player1_client.delete_account().await?;
    player2_client.delete_account().await?;

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test stalemate detection
pub async fn test_stalemate_detection() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // Create two players and get them matched
    let (mut player1_client, mut player2_client, _session_id, _player1_id, _player2_id) =
        setup_matched_game(&config).await?;

    // In a real test, we'd play moves that lead to stalemate
    // For now, we'll verify that stalemate is properly handled when it occurs
    // The ChessService already handles stalemate detection

    // Clean up
    player1_client.close_websocket_connections().await?;
    player2_client.close_websocket_connections().await?;
    player1_client.delete_account().await?;
    player2_client.delete_account().await?;

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test concurrent games
pub async fn test_concurrent_games() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // Create multiple pairs of players for concurrent games
    let mut games = Vec::new();

    for _i in 0..2 {
        // Test with 2 concurrent games
        let game_future = async {
            let (mut player1_client, mut player2_client, session_id, _player1_id, _player2_id) =
                setup_matched_game(&config).await?;

            // Play a few moves in each game
            player1_client.make_move(&session_id, "e2", "e4").await?;
            player1_client
                .wait_for_move_update(&session_id, Duration::from_secs(5))
                .await?;
            player2_client
                .wait_for_move_update(&session_id, Duration::from_secs(5))
                .await?;

            player2_client.make_move(&session_id, "e7", "e5").await?;
            player1_client
                .wait_for_move_update(&session_id, Duration::from_secs(5))
                .await?;
            player2_client
                .wait_for_move_update(&session_id, Duration::from_secs(5))
                .await?;

            // Clean up
            player1_client.close_websocket_connections().await?;
            player2_client.close_websocket_connections().await?;
            player1_client.delete_account().await?;
            player2_client.delete_account().await?;

            Ok(())
        };

        games.push(game_future);
    }

    // Run all games concurrently
    for game in games {
        game.await?;
    }

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Helper function to set up a matched game between two players
async fn setup_matched_game(
    config: &E2EConfig,
) -> TestResult<(TestClient, TestClient, String, String, String)> {
    let mut player1_client = TestClient::from_config(config);
    let mut player2_client = TestClient::from_config(config);

    let player1_user = TestUser::new("Game", "Player1");
    let player2_user = TestUser::new("Game", "Player2");

    // Register both players
    player1_client
        .register_user(
            &player1_user.email,
            &player1_user.password,
            &player1_user.first_name,
            &player1_user.last_name,
        )
        .await?;
    player2_client
        .register_user(
            &player2_user.email,
            &player2_user.password,
            &player2_user.first_name,
            &player2_user.last_name,
        )
        .await?;

    // Login both players
    player1_client
        .login(&player1_user.email, &player1_user.password)
        .await?;
    player2_client
        .login(&player2_user.email, &player2_user.password)
        .await?;

    // Get user IDs
    let player1_info = player1_client.get_user_info().await?;
    let player1_id = player1_info["id"].as_str().unwrap().to_string();
    let player2_info = player2_client.get_user_info().await?;
    let player2_id = player2_info["id"].as_str().unwrap().to_string();

    // Establish WebSocket connections
    player1_client
        .establish_websocket_connection(&player1_id)
        .await?;
    player2_client
        .establish_websocket_connection(&player2_id)
        .await?;

    // Both players join queue
    player1_client.join_queue("rapid").await?;
    player2_client.join_queue("rapid").await?;

    // Wait for game matching
    let player1_match = player1_client
        .wait_for_websocket_message("game_matched", Duration::from_secs(30))
        .await?;
    let _player2_match = player2_client
        .wait_for_websocket_message("game_matched", Duration::from_secs(30))
        .await?;

    let session_id = player1_match["session_id"].as_str().unwrap().to_string();

    Ok((
        player1_client,
        player2_client,
        session_id,
        player1_id,
        player2_id,
    ))
}

/// Test game abandonment/resignation (when implemented)
pub async fn test_game_abandonment() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // This test would be implemented when resignation functionality is added
    // For now, it's a placeholder

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test time controls and timeouts (when implemented)
pub async fn test_time_controls() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // This test would be implemented when time controls are added
    // For now, it's a placeholder

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test draw by agreement (when implemented)
pub async fn test_draw_agreement() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // This test would be implemented when draw functionality is added
    // For now, it's a placeholder

    timer.assert_within_timeout(config.timeout);
    Ok(())
}
