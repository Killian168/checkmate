use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinQueueMessage {
    pub action: String, // "join_queue"
    pub time_control: String,
    pub min_rating: Option<i32>,
    pub max_rating: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaveQueueMessage {
    pub action: String, // "leave_queue"
    pub time_control: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub status: String,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    pub queue_key: String,
    pub user_id: String,
    pub time_control: String,
    pub rating_bucket: String,
    pub rating: i32,
    pub joined_at: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_at: Option<String>,
    pub min_rating: Option<i32>,
    pub max_rating: Option<i32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Connection {
    pub connection_id: String,
    pub user_id: String,
    pub connected_at: String,
}
