use std::sync::Arc;
use tracing::info;

use crate::repositories::websocket_repository::WebSocketRepository;

#[derive(Clone)]
pub struct WebSocketService {
    repository: Arc<dyn WebSocketRepository>,
}

impl WebSocketService {
    pub fn new(repository: Arc<dyn WebSocketRepository>) -> Self {
        Self { repository }
    }

    pub async fn store_connection(
        &self,
        player_id: &str,
        connection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Storing WebSocket connection for player: {}", player_id);
        self.repository
            .store_connection(player_id, connection_id)
            .await
    }

    pub async fn remove_connection(
        &self,
        player_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Removing WebSocket connection for player: {}", player_id);
        self.repository.remove_connection(player_id).await
    }

    pub async fn remove_connection_by_id(
        &self,
        connection_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Removing WebSocket connection by ID: {}", connection_id);
        self.repository.remove_connection_by_id(connection_id).await
    }

    pub async fn get_connection_id(
        &self,
        player_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.repository.get_connection_id(player_id).await
    }

    pub async fn send_notification(
        &self,
        player_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(connection_id) = self.get_connection_id(player_id).await? {
            info!("Sending notification to player {}: {}", player_id, message);
            // This would be implemented in the repository to use API Gateway Management API
            self.repository
                .send_message(&connection_id, message)
                .await?;
        } else {
            info!(
                "Player {} is not connected, skipping notification",
                player_id
            );
        }
        Ok(())
    }

    pub async fn send_message(
        &self,
        connection_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Sending message to connection: {}", connection_id);
        self.repository.send_message(connection_id, message).await
    }
}
