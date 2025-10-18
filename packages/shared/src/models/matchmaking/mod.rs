pub mod requests;
pub mod responses;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a player currently in the matchmaking queue.
/// Each record corresponds to a DynamoDB item, partitioned by queue type & rating bucket.
/// Example PK: "rapid#1400", SK: "player-uuid"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatchmakingUser {
    pub queue_rating: String,
    pub player_id: String,
    pub joined_at: DateTime<Utc>,
}

impl MatchmakingUser {
    pub fn new(player_id: &str, rating: i32, queue_type: &str) -> Self {
        let rating_bucket = (rating / 100) * 100;
        let queue_rating = format!("{}#{}", queue_type, rating_bucket);

        MatchmakingUser {
            queue_rating,
            player_id: player_id.to_string(),
            joined_at: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub enum QueueRepositoryError {
    NotFound,
    AlreadyExists,
    Serialization(String),
    DynamoDb(String),
}

impl std::fmt::Display for QueueRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueRepositoryError::NotFound => write!(f, "MatchmakingUser not found"),
            QueueRepositoryError::AlreadyExists => {
                write!(f, "MatchmakingUser already exists")
            }
            QueueRepositoryError::Serialization(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            QueueRepositoryError::DynamoDb(msg) => write!(f, "DynamoDB error: {}", msg),
        }
    }
}

impl std::error::Error for QueueRepositoryError {}

#[async_trait]
pub trait QueueRepository {
    async fn join_queue(&self, user: &MatchmakingUser) -> Result<(), QueueRepositoryError>;
    async fn leave_queue(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<(), QueueRepositoryError>;
}
