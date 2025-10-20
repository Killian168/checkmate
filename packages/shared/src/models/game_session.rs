use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
