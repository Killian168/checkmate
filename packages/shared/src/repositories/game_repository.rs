use crate::models::game_session::GameSession;
use crate::repositories::errors::game_repository_errors::GameSessionRepositoryError;
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;

pub struct DynamoDbGameSessionRepository {
    pub client: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

impl DynamoDbGameSessionRepository {
    pub fn new(client: Client) -> Self {
        let table_name = std::env::var("GAME_SESSIONS_TABLE")
            .expect("GAME_SESSIONS_TABLE environment variable must be set");
        Self { client, table_name }
    }
}

#[async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn create_game_session(
        &self,
        game_session: &GameSession,
    ) -> Result<(), GameSessionRepositoryError>;
}

#[async_trait]
impl GameSessionRepository for DynamoDbGameSessionRepository {
    async fn create_game_session(
        &self,
        game_session: &GameSession,
    ) -> Result<(), GameSessionRepositoryError> {
        let item = serde_dynamo::to_item(game_session)
            .map_err(|e| GameSessionRepositoryError::Serialization(e.to_string()))?;

        let request = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item));

        request
            .send()
            .await
            .map_err(|e| GameSessionRepositoryError::DynamoDb(e.to_string()))?;

        Ok(())
    }
}
