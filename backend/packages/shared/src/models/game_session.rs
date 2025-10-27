use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Ongoing,
    Checkmate,
    Stalemate,
    Resigned,
    Draw,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
