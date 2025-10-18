use async_trait::async_trait;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::types::{AttributeValue, TransactWriteItem};
use serde_dynamo::aws_sdk_dynamodb_1::from_item;
use shared::models::matchmaking::MatchmakingUser;

use crate::models::GameSession;

#[derive(Debug)]
pub enum MatchmakingRepositoryError {
    Serialization(String),
    DynamoDb(String),
    TransactionError(String),
}

impl std::fmt::Display for MatchmakingRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchmakingRepositoryError::Serialization(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            MatchmakingRepositoryError::DynamoDb(msg) => write!(f, "DynamoDB error: {}", msg),
            MatchmakingRepositoryError::TransactionError(msg) => {
                write!(f, "Transaction error: {}", msg)
            }
        }
    }
}

impl std::error::Error for MatchmakingRepositoryError {}

#[async_trait]
pub trait MatchmakingRepository: Send + Sync {
    async fn find_potential_opponents(
        &self,
        queue_rating: &str,
        excluded_player_id: &str,
    ) -> Result<Vec<MatchmakingUser>, MatchmakingRepositoryError>;

    async fn create_game_session(
        &self,
        player1: &MatchmakingUser,
        player2: &MatchmakingUser,
        game_session: &GameSession,
    ) -> Result<(), MatchmakingRepositoryError>;

    async fn check_and_reserve_opponent(
        &self,
        opponent: &MatchmakingUser,
    ) -> Result<bool, MatchmakingRepositoryError>;
}

pub struct DynamoDbMatchmakingRepository {
    pub client: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

impl DynamoDbMatchmakingRepository {
    pub fn new(client: aws_sdk_dynamodb::Client) -> Self {
        let table_name = std::env::var("MATCHMAKING_TABLE")
            .expect("MATCHMAKING_TABLE environment variable must be set");
        Self { client, table_name }
    }
}

#[async_trait]
impl MatchmakingRepository for DynamoDbMatchmakingRepository {
    async fn find_potential_opponents(
        &self,
        queue_rating: &str,
        excluded_player_id: &str,
    ) -> Result<Vec<MatchmakingUser>, MatchmakingRepositoryError> {
        let query_result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("queue_rating = :queue_rating")
            .expression_attribute_values(
                ":queue_rating",
                AttributeValue::S(queue_rating.to_string()),
            )
            .send()
            .await
            .map_err(|e| MatchmakingRepositoryError::DynamoDb(e.to_string()))?;

        let mut opponents = Vec::new();

        if let Some(items) = query_result.items {
            for item in items {
                let opponent: MatchmakingUser = from_item(item)
                    .map_err(|e| MatchmakingRepositoryError::Serialization(e.to_string()))?;

                // Skip the excluded player
                if opponent.player_id != excluded_player_id {
                    opponents.push(opponent);
                }
            }
        }

        // Sort opponents by join time (oldest first) to prioritize longest waiting players
        opponents.sort_by_key(|opponent| opponent.joined_at);

        Ok(opponents)
    }

    async fn create_game_session(
        &self,
        player1: &MatchmakingUser,
        player2: &MatchmakingUser,
        _game_session: &GameSession,
    ) -> Result<(), MatchmakingRepositoryError> {
        // Use DynamoDB transaction to atomically remove both players
        let transaction_items = vec![
            // Remove player1 from queue
            TransactWriteItem::builder()
                .delete(
                    aws_sdk_dynamodb::types::Delete::builder()
                        .table_name(&self.table_name)
                        .key(
                            "queue_rating",
                            AttributeValue::S(player1.queue_rating.clone()),
                        )
                        .key("player_id", AttributeValue::S(player1.player_id.clone()))
                        .build()
                        .map_err(|e| MatchmakingRepositoryError::TransactionError(e.to_string()))?,
                )
                .build(),
            // Remove player2 from queue
            TransactWriteItem::builder()
                .delete(
                    aws_sdk_dynamodb::types::Delete::builder()
                        .table_name(&self.table_name)
                        .key(
                            "queue_rating",
                            AttributeValue::S(player2.queue_rating.clone()),
                        )
                        .key("player_id", AttributeValue::S(player2.player_id.clone()))
                        .build()
                        .map_err(|e| MatchmakingRepositoryError::TransactionError(e.to_string()))?,
                )
                .build(),
        ];

        // Execute the transaction
        self.client
            .transact_write_items()
            .set_transact_items(Some(transaction_items))
            .send()
            .await
            .map_err(|e| MatchmakingRepositoryError::TransactionError(e.to_string()))?;

        Ok(())
    }

    async fn check_and_reserve_opponent(
        &self,
        opponent: &MatchmakingUser,
    ) -> Result<bool, MatchmakingRepositoryError> {
        // Try to update the opponent with a "reserved" flag atomically
        // This prevents other matchmaking processes from selecting the same opponent
        let update_result = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key(
                "queue_rating",
                AttributeValue::S(opponent.queue_rating.clone()),
            )
            .key("player_id", AttributeValue::S(opponent.player_id.clone()))
            .update_expression("SET reserved = :reserved")
            .condition_expression("attribute_not_exists(reserved)")
            .expression_attribute_values(":reserved", AttributeValue::Bool(true))
            .send()
            .await;

        match update_result {
            Ok(_) => Ok(true), // Successfully reserved the opponent
            Err(e) => {
                // Check if the condition check failed (opponent already reserved)
                if let SdkError::ServiceError(service_err) = &e {
                    if service_err.err().is_conditional_check_failed_exception() {
                        return Ok(false); // Opponent already reserved by another process
                    }
                }
                Err(MatchmakingRepositoryError::DynamoDb(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // Mock repository for testing
    #[derive(Clone)]
    pub struct MockMatchmakingRepository {
        pub opponents: Vec<MatchmakingUser>,
        pub create_game_session_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockMatchmakingRepository {
        pub fn new() -> Self {
            Self {
                opponents: Vec::new(),
                create_game_session_called: std::sync::Arc::new(
                    std::sync::atomic::AtomicBool::new(false),
                ),
            }
        }

        pub fn with_opponents(mut self, opponents: Vec<MatchmakingUser>) -> Self {
            self.opponents = opponents;
            self
        }
    }

    #[async_trait]
    impl MatchmakingRepository for MockMatchmakingRepository {
        async fn find_potential_opponents(
            &self,
            queue_rating: &str,
            excluded_player_id: &str,
        ) -> Result<Vec<MatchmakingUser>, MatchmakingRepositoryError> {
            let mut opponents: Vec<MatchmakingUser> = self
                .opponents
                .iter()
                .filter(|opponent| opponent.player_id != excluded_player_id)
                .filter(|opponent| opponent.queue_rating == queue_rating)
                .cloned()
                .collect();

            // Sort opponents by join time (oldest first) to prioritize longest waiting players
            opponents.sort_by_key(|opponent| opponent.joined_at);

            Ok(opponents)
        }

        async fn create_game_session(
            &self,
            _player1: &MatchmakingUser,
            _player2: &MatchmakingUser,
            _game_session: &GameSession,
        ) -> Result<(), MatchmakingRepositoryError> {
            self.create_game_session_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn check_and_reserve_opponent(
            &self,
            _opponent: &MatchmakingUser,
        ) -> Result<bool, MatchmakingRepositoryError> {
            // Mock always succeeds in reserving opponent
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_find_potential_opponents_excludes_current_user() {
        let now = chrono::Utc::now();
        let user1 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: now,
        };

        let user2 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(5),
        };

        let repository =
            MockMatchmakingRepository::new().with_opponents(vec![user1.clone(), user2.clone()]);

        let opponents = repository
            .find_potential_opponents("rapid#1400", "user1")
            .await
            .unwrap();

        assert_eq!(opponents.len(), 1);
        assert_eq!(opponents[0].player_id, "user2");
    }

    #[tokio::test]
    async fn test_find_potential_opponents_sorts_by_join_time() {
        let now = chrono::Utc::now();
        let user1 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: now - chrono::Duration::minutes(10), // Oldest
        };

        let user2 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(5), // Middle
        };

        let user3 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user3".to_string(),
            joined_at: now, // Newest
        };

        let repository = MockMatchmakingRepository::new().with_opponents(vec![
            user3.clone(),
            user1.clone(),
            user2.clone(),
        ]);

        let opponents = repository
            .find_potential_opponents("rapid#1400", "nonexistent")
            .await
            .unwrap();

        assert_eq!(opponents.len(), 3);
        // Should be sorted by join time (oldest first)
        assert_eq!(opponents[0].player_id, "user1"); // Oldest
        assert_eq!(opponents[1].player_id, "user2"); // Middle
        assert_eq!(opponents[2].player_id, "user3"); // Newest
    }

    #[tokio::test]
    async fn test_create_game_session_sets_flag() {
        let repository = MockMatchmakingRepository::new();
        let user1 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: chrono::Utc::now(),
        };
        let user2 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user2".to_string(),
            joined_at: chrono::Utc::now(),
        };
        let game_session = GameSession::new("user1", "user2", 1400, 1400, "rapid");

        repository
            .create_game_session(&user1, &user2, &game_session)
            .await
            .unwrap();

        assert!(repository
            .create_game_session_called
            .load(std::sync::atomic::Ordering::SeqCst));
    }
}
