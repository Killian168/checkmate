use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueueEntry {
    pub queue_key: String,
    pub user_id: String,
    pub time_control: String,
    pub rating: i32,
    pub joined_at: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GameMatchedMessage {
    pub action: String,
    pub game_id: String,
    pub opponent_id: String,
    pub color: String,
    pub time_control: String,
}

#[derive(Debug, Deserialize)]
pub struct Connection {
    pub connection_id: String,
    pub user_id: String,
}
