use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_session_creation() {
        let player1_id = "player1-uuid";
        let player2_id = "player2-uuid";

        let session = GameSession::new(player1_id, player2_id);

        assert_eq!(session.player1_id, player1_id);
        assert_eq!(session.player2_id, player2_id);
        assert!(!session.session_id.is_empty());
    }

    #[test]
    fn test_game_session_id_uniqueness() {
        let player1_id = "player1";
        let player2_id = "player2";

        let session1 = GameSession::new(player1_id, player2_id);
        let session2 = GameSession::new(player1_id, player2_id);

        assert_ne!(session1.session_id, session2.session_id);
        assert_eq!(session1.player1_id, session2.player1_id);
        assert_eq!(session1.player2_id, session2.player2_id);
    }

    #[test]
    fn test_game_session_serialization() {
        let session = GameSession::new("player1", "player2");

        // Test serialization
        let serialized = serde_json::to_string(&session).unwrap();
        assert!(serialized.contains("player1"));
        assert!(serialized.contains("player2"));
        assert!(serialized.contains("session_id"));

        // Test deserialization
        let deserialized: GameSession = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.player1_id, session.player1_id);
        assert_eq!(deserialized.player2_id, session.player2_id);
        assert_eq!(deserialized.session_id, session.session_id);
    }

    #[test]
    fn test_game_session_different_players() {
        let player1_id = "unique-player-1";
        let player2_id = "unique-player-2";

        let session = GameSession::new(player1_id, player2_id);

        assert_ne!(session.player1_id, session.player2_id);
        assert_eq!(session.player1_id, player1_id);
        assert_eq!(session.player2_id, player2_id);
    }

    #[test]
    fn test_game_session_string_conversion() {
        let player1_id = "string-player-1";
        let player2_id = "string-player-2";

        let session = GameSession::new(player1_id, player2_id);

        // Ensure string conversion works correctly
        assert_eq!(session.player1_id.as_str(), player1_id);
        assert_eq!(session.player2_id.as_str(), player2_id);
    }

    #[test]
    fn test_game_session_clone() {
        let session = GameSession::new("player1", "player2");
        let cloned = session.clone();

        assert_eq!(session.session_id, cloned.session_id);
        assert_eq!(session.player1_id, cloned.player1_id);
        assert_eq!(session.player2_id, cloned.player2_id);
    }

    #[test]
    fn test_game_session_debug_format() {
        let session = GameSession::new("debug-player-1", "debug-player-2");

        let debug_output = format!("{:?}", session);

        assert!(debug_output.contains("GameSession"));
        assert!(debug_output.contains("session_id"));
        assert!(debug_output.contains("player1_id"));
        assert!(debug_output.contains("player2_id"));
    }

    #[test]
    fn test_multiple_game_sessions() {
        let players = vec![
            ("player-a", "player-b"),
            ("player-c", "player-d"),
            ("player-e", "player-f"),
        ];

        let sessions: Vec<GameSession> = players
            .iter()
            .map(|(p1, p2)| GameSession::new(p1, p2))
            .collect();

        assert_eq!(sessions.len(), 3);

        // Verify all sessions have unique IDs
        let session_ids: Vec<&str> = sessions.iter().map(|s| s.session_id.as_str()).collect();
        let unique_ids: std::collections::HashSet<&str> = session_ids.iter().cloned().collect();
        assert_eq!(unique_ids.len(), 3, "All session IDs should be unique");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub session_id: String,
    pub player1_id: String,
    pub player2_id: String,
}

impl GameSession {
    pub fn new(player1_id: &str, player2_id: &str) -> Self {
        GameSession {
            session_id: Uuid::new_v4().to_string(),
            player1_id: player1_id.to_string(),
            player2_id: player2_id.to_string(),
        }
    }
}
