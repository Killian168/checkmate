use crate::models::queue::QueueUser;
use crate::repositories::errors::queue_repository_errors::QueueRepositoryError;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde_dynamo::{from_item, to_item};

pub struct DynamoDbQueueRepository {
    pub client: Client,
    pub table_name: String,
}

impl DynamoDbQueueRepository {
    pub fn new(client: Client) -> Self {
        let table_name =
            std::env::var("QUEUE_TABLE").expect("QUEUE_TABLE environment variable must be set");
        Self { client, table_name }
    }
}

#[async_trait]
pub trait QueueRepository {
    async fn join_queue(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<(), QueueRepositoryError>;
    async fn leave_queue(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<(), QueueRepositoryError>;
    async fn find_potential_opponents(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<Vec<QueueUser>, QueueRepositoryError>;
    async fn reserve_opponent(&self, opponent: &QueueUser) -> Result<bool, QueueRepositoryError>;
}

#[async_trait]
impl QueueRepository for DynamoDbQueueRepository {
    async fn join_queue(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<(), QueueRepositoryError> {
        let user = QueueUser::new(player_id, rating, queue_type);
        let item =
            to_item(&user).map_err(|e| QueueRepositoryError::Serialization(e.to_string()))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| QueueRepositoryError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    async fn leave_queue(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<(), QueueRepositoryError> {
        let rating_bucket = (rating / 100) * 100;
        let queue_rating = format!("{}#{}", queue_type, rating_bucket);

        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("queue_rating", AttributeValue::S(queue_rating))
            .key("player_id", AttributeValue::S(player_id.to_string()))
            .send()
            .await
            .map_err(|e| QueueRepositoryError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    async fn find_potential_opponents(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<Vec<QueueUser>, QueueRepositoryError> {
        let rating_bucket = (rating / 100) * 100;
        let queue_rating = format!("{}#{}", queue_type, rating_bucket);

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
            .map_err(|e| QueueRepositoryError::DynamoDb(e.to_string()))?;

        let mut opponents = Vec::new();

        if let Some(items) = query_result.items {
            for item in items {
                let opponent: QueueUser = from_item(item)
                    .map_err(|e| QueueRepositoryError::Serialization(e.to_string()))?;

                // Skip the excluded player
                if opponent.player_id != player_id {
                    opponents.push(opponent);
                }
            }
        }

        Ok(opponents)
    }

    async fn reserve_opponent(&self, opponent: &QueueUser) -> Result<bool, QueueRepositoryError> {
        // Instead of using a reserved attribute, we'll delete the opponent from the queue
        // This ensures atomic reservation - if we can delete it, we've successfully reserved it
        let delete_result = self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key(
                "queue_rating",
                AttributeValue::S(opponent.queue_rating.clone()),
            )
            .key("player_id", AttributeValue::S(opponent.player_id.clone()))
            .condition_expression("attribute_exists(player_id)")
            .send()
            .await;

        match delete_result {
            Ok(_) => Ok(true), // Successfully reserved (deleted) the opponent
            Err(e) => {
                // Check if the condition check failed (opponent already deleted/reserved)
                if let SdkError::ServiceError(service_err) = &e {
                    if service_err.err().is_conditional_check_failed_exception() {
                        return Ok(false); // Opponent already reserved by another process
                    }
                }
                Err(QueueRepositoryError::DynamoDb(e.to_string()))
            }
        }
    }
}
