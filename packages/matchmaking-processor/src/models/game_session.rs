use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub session_id: String,
    pub player1_id: String,
    pub player2_id: String,
    pub player1_rating: i32,
    pub player2_rating: i32,
    pub queue_type: String,
    pub created_at: String,
}

impl GameSession {
    pub fn new(player1_id: &str, player2_id: &str, player1_rating: i32, player2_rating: i32, queue_type: &str) -> Self {
        GameSession {
            session_id: Uuid::new_v4().to_string(),
            player1_id: player1_id.to_string(),
            player2_id: player2_id.to_string(),
            player1_rating,
            player2_rating,
            queue_type: queue_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
