pub mod requests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a player currently in the matchmaking queue.
/// Each record corresponds to a DynamoDB item, partitioned by queue type & rating bucket.
/// Example PK: "rapid#1400", SK: "player-uuid"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueUser {
    pub queue_rating: String,
    pub player_id: String,
    pub joined_at: DateTime<Utc>,
}

impl QueueUser {
    pub fn new(player_id: &str, rating: i32, queue_type: &str) -> Self {
        let rating_bucket = (rating / 100) * 100;
        let queue_rating = format!("{}#{}", queue_type, rating_bucket);

        QueueUser {
            queue_rating,
            player_id: player_id.to_string(),
            joined_at: Utc::now(),
        }
    }
    /// Extracts the rating value from the queue rating string (e.g., "rapid#1400" -> 1400)
    pub fn rating(&self) -> i32 {
        self.queue_rating
            .split('#')
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Extracts the queue type from the queue rating string (e.g., "rapid#1400" -> "rapid")
    pub fn queue_type(&self) -> String {
        self.queue_rating
            .split('#')
            .next()
            .unwrap_or("")
            .to_string()
    }
}
