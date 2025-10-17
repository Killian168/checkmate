use crate::models::matchmaking::{
    MatchmakingUser, MatchmakingUserRepository, MatchmakingUserRepositoryError,
};
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use serde_dynamo::aws_sdk_dynamodb_1::{from_item, to_item};

pub struct DynamoDbMatchmakingUserRepository {
    pub client: Client,
    pub table_name: String,
}

impl DynamoDbMatchmakingUserRepository {
    pub fn new(client: Client) -> Self {
        let table_name = std::env::var("MATCHMAKING_TABLE")
            .expect("MATCHMAKING_TABLE environment variable must be set");
        Self { client, table_name }
    }
}

#[async_trait]
impl MatchmakingUserRepository for DynamoDbMatchmakingUserRepository {
    async fn join_queue(
        &self,
        user: &MatchmakingUser,
    ) -> Result<(), MatchmakingUserRepositoryError> {
        let item = to_item(user)
            .map_err(|e| MatchmakingUserRepositoryError::Serialization(e.to_string()))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| MatchmakingUserRepositoryError::DynamoDb(e.to_string()))?;

        Ok(())
    }

    async fn leave_queue(
        &self,
        player_id: &str,
        queue_type: &str,
        rating: i32,
    ) -> Result<(), MatchmakingUserRepositoryError> {
        let rating_bucket = (rating / 100) * 100;
        let queue_rating = format!("{}#{}", queue_type, rating_bucket);

        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key(
                "queue_rating",
                aws_sdk_dynamodb::types::AttributeValue::S(queue_rating),
            )
            .key(
                "player_id",
                aws_sdk_dynamodb::types::AttributeValue::S(player_id.to_string()),
            )
            .send()
            .await
            .map_err(|e| MatchmakingUserRepositoryError::DynamoDb(e.to_string()))?;

        Ok(())
    }
}
