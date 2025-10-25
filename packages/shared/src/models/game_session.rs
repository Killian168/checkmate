use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameStatus {
    Ongoing,
    Checkmate,
    Stalemate,
    Resigned,
    Draw,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Turn {
    White,
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub session_id: String,
    pub player1_id: String,
    pub player2_id: String,
    pub fen_board: String,
    pub player_white_id: String,
    pub player_black_id: String,
    pub move_history: Vec<String>,
    pub status: GameStatus,
    pub winner: Option<String>,
    pub creation_time: DateTime<Utc>,
    pub whose_turn: Turn,
    pub time_remaining_white: u64,
    pub time_remaining_black: u64,
}

impl GameSession {
    pub fn new(player1_id: &str, player2_id: &str) -> Self {
        GameSession {
            session_id: Uuid::new_v4().to_string(),
            player1_id: player1_id.to_string(),
            player2_id: player2_id.to_string(),
            fen_board: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            player_white_id: player1_id.to_string(),
            player_black_id: player2_id.to_string(),
            move_history: vec![],
            status: GameStatus::Ongoing,
            winner: None,
            creation_time: Utc::now(),
            whose_turn: Turn::White,
            time_remaining_white: 600,
            time_remaining_black: 600,
        }
    }
}

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

    #[test]
    fn test_new_game_session_fields() {
        let session = GameSession::new("player1", "player2");

        assert_eq!(
            session.fen_board,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
        assert_eq!(session.player_white_id, "player1");
        assert_eq!(session.player_black_id, "player2");
        assert!(session.move_history.is_empty());
        assert!(matches!(session.status, GameStatus::Ongoing));
        assert!(session.winner.is_none());
        assert!(matches!(session.whose_turn, Turn::White));
        assert_eq!(session.time_remaining_white, 600);
        assert_eq!(session.time_remaining_black, 600);

        // creation_time should be recent
        let now = Utc::now();
        assert!((now - session.creation_time).num_seconds() < 10);
    }

    #[test]
    fn test_enum_serialization() {
        let status = GameStatus::Checkmate;
        let turn = Turn::Black;

        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"Checkmate\"");

        let deserialized: GameStatus = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, GameStatus::Checkmate));

        let turn_serialized = serde_json::to_string(&turn).unwrap();
        assert_eq!(turn_serialized, "\"Black\"");
    }

    #[test]
    fn test_serialization_with_new_fields() {
        let session = GameSession::new("player1", "player2");

        let serialized = serde_json::to_string(&session).unwrap();

        assert!(serialized.contains("\"fen_board\""));
        assert!(serialized.contains("\"player_white_id\""));
        assert!(serialized.contains("\"status\""));
        assert!(serialized.contains("\"time_remaining_white\""));

        let deserialized: GameSession = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.session_id, session.session_id);
        assert_eq!(deserialized.fen_board, session.fen_board);
    }
}
