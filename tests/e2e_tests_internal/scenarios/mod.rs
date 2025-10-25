pub mod authentication;
pub mod game;
pub mod queue;
pub mod websocket;

/// Test suite categories for organization
pub enum TestCategory {
    /// Authentication and user management tests
    Authentication,
    /// Queue system functionality tests
    Queue,
    /// WebSocket connection and messaging tests
    WebSocket,
    /// Chess game functionality tests
    Game,
}

impl TestCategory {
    /// Get all test functions for this category
    pub fn test_functions(&self) -> Vec<&'static str> {
        match self {
            TestCategory::Authentication => vec![
                "test_complete_user_journey",
                "test_authentication_flow",
                "test_error_scenarios",
                "test_session_persistence",
                "test_duplicate_registration",
                "test_token_expiration",
            ],
            TestCategory::Queue => vec![
                "test_multiple_users_queue_interaction",
                "test_concurrent_queue_operations",
                "test_different_queue_types",
                "test_queue_rejoining",
                "test_queue_operations_without_leaving",
                "test_queue_timeout_scenarios",
            ],
            TestCategory::WebSocket => vec![
                "test_websocket_connection_flow",
                "test_websocket_multiple_connections",
                "test_websocket_reconnection",
                "test_websocket_message_exchange",
                "test_websocket_connection_timeout",
                "test_websocket_invalid_credentials",
                "test_concurrent_websocket_connections",
            ],
            TestCategory::Game => vec![
                "test_complete_chess_game",
                "test_invalid_move_handling",
                "test_pawn_promotion",
                "test_stalemate_detection",
                "test_concurrent_games",
                "test_game_abandonment",
                "test_time_controls",
                "test_draw_agreement",
            ],
        }
    }

    /// Get the number of tests in this category
    pub fn test_count(&self) -> usize {
        self.test_functions().len()
    }
}
