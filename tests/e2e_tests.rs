mod e2e_tests_internal;

use e2e_tests_internal::scenarios::TestCategory;
pub use e2e_tests_internal::{
    run_e2e_tests_by_category, E2EConfig, TestClient, TestResult, TestTimer, TestUser,
};

/// Complete chess game test from start to finish
#[tokio::test]
async fn run_complete_chess_game() {
    e2e_tests_internal::scenarios::game::test_complete_chess_game()
        .await
        .expect("Complete chess game test failed");
}

/// Invalid move handling test
#[tokio::test]
async fn run_invalid_move_handling() {
    e2e_tests_internal::scenarios::game::test_invalid_move_handling()
        .await
        .expect("Invalid move handling test failed");
}

/// Pawn promotion test
#[tokio::test]
async fn run_pawn_promotion() {
    e2e_tests_internal::scenarios::game::test_pawn_promotion()
        .await
        .expect("Pawn promotion test failed");
}

/// Stalemate detection test
#[tokio::test]
async fn run_stalemate_detection() {
    e2e_tests_internal::scenarios::game::test_stalemate_detection()
        .await
        .expect("Stalemate detection test failed");
}

/// Concurrent games test
#[tokio::test]
async fn run_concurrent_games() {
    e2e_tests_internal::scenarios::game::test_concurrent_games()
        .await
        .expect("Concurrent games test failed");
}

/// Game abandonment test (placeholder)
#[tokio::test]
async fn run_game_abandonment() {
    e2e_tests_internal::scenarios::game::test_game_abandonment()
        .await
        .expect("Game abandonment test failed");
}

/// Time controls test (placeholder)
#[tokio::test]
async fn run_time_controls() {
    e2e_tests_internal::scenarios::game::test_time_controls()
        .await
        .expect("Time controls test failed");
}

/// Draw agreement test (placeholder)
#[tokio::test]
async fn run_draw_agreement() {
    e2e_tests_internal::scenarios::game::test_draw_agreement()
        .await
        .expect("Draw agreement test failed");
}

/// Run all tests in game category
#[tokio::test]
async fn run_game_category_tests() {
    run_e2e_tests_by_category(TestCategory::Game)
        .await
        .expect("Game category tests failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use e2e_tests_internal::scenarios::TestCategory;

    /// Complete user journey test - registration to deletion
    #[tokio::test]
    async fn run_complete_user_journey() {
        e2e_tests_internal::scenarios::authentication::test_complete_user_journey()
            .await
            .expect("Complete user journey test failed");
    }

    /// WebSocket connection flow test
    #[tokio::test]
    async fn run_websocket_connection_flow() {
        e2e_tests_internal::scenarios::websocket::test_websocket_connection_flow()
            .await
            .expect("WebSocket connection flow test failed");
    }

    /// Multiple WebSocket connections test
    #[tokio::test]
    async fn run_websocket_multiple_connections() {
        e2e_tests_internal::scenarios::websocket::test_websocket_multiple_connections()
            .await
            .expect("WebSocket multiple connections test failed");
    }

    /// WebSocket reconnection test
    #[tokio::test]
    async fn run_websocket_reconnection() {
        e2e_tests_internal::scenarios::websocket::test_websocket_reconnection()
            .await
            .expect("WebSocket reconnection test failed");
    }

    /// Multiple users queue interaction test
    #[tokio::test]
    async fn run_multiple_users_queue_interaction() {
        e2e_tests_internal::scenarios::queue::test_multiple_users_queue_interaction()
            .await
            .expect("Multiple users queue interaction test failed");
    }

    /// Game matching notifications test
    #[tokio::test]
    async fn run_game_matching_notifications() {
        e2e_tests_internal::scenarios::websocket::test_game_matching_notifications()
            .await
            .expect("Game matching notifications test failed");
    }

    /// Authentication flow test
    #[tokio::test]
    async fn run_authentication_flow() {
        e2e_tests_internal::scenarios::authentication::test_authentication_flow()
            .await
            .expect("Authentication flow test failed");
    }

    /// Error scenarios test
    #[tokio::test]
    async fn run_error_scenarios() {
        e2e_tests_internal::scenarios::authentication::test_error_scenarios()
            .await
            .expect("Error scenarios test failed");
    }

    /// Concurrent queue operations test
    #[tokio::test]
    async fn run_concurrent_queue_operations() {
        e2e_tests_internal::scenarios::queue::test_concurrent_queue_operations()
            .await
            .expect("Concurrent queue operations test failed");
    }

    /// Session persistence test
    #[tokio::test]
    async fn run_session_persistence() {
        e2e_tests_internal::scenarios::authentication::test_session_persistence()
            .await
            .expect("Session persistence test failed");
    }

    /// Run all tests in a specific category
    #[tokio::test]
    async fn run_authentication_category_tests() {
        run_e2e_tests_by_category(TestCategory::Authentication)
            .await
            .expect("Authentication category tests failed");
    }

    /// Run all tests in queue category
    #[tokio::test]
    async fn run_queue_category_tests() {
        run_e2e_tests_by_category(TestCategory::Queue)
            .await
            .expect("Queue category tests failed");
    }

    /// Run all tests in WebSocket category
    #[tokio::test]
    async fn run_websocket_category_tests() {
        run_e2e_tests_by_category(TestCategory::WebSocket)
            .await
            .expect("WebSocket category tests failed");
    }
}
