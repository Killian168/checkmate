use std::sync::Arc;

use crate::services::errors::matchmaking_service_errors::MatchmakingServiceError;
use shared::models::matchmaking::{MatchmakingUser, MatchmakingUserRepository};
use shared::models::user::User;

pub struct MatchmakingService {
    repository: Arc<dyn MatchmakingUserRepository + Send + Sync>,
}

impl MatchmakingService {
    pub fn new(repository: Arc<dyn MatchmakingUserRepository + Send + Sync>) -> Self {
        MatchmakingService { repository }
    }

    pub async fn join_queue(
        &self,
        player: &User,
        queue_type: &str,
    ) -> Result<MatchmakingUser, MatchmakingServiceError> {
        let player_id = &player.id;

        if player_id.is_empty() {
            return Err(MatchmakingServiceError::ValidationError(
                "Player ID cannot be empty".to_string(),
            ));
        }

        if queue_type.is_empty() {
            return Err(MatchmakingServiceError::ValidationError(
                "Queue type cannot be empty".to_string(),
            ));
        }

        let user = MatchmakingUser::new(player_id, player.rating, queue_type);

        self.repository
            .join_queue(&user)
            .await
            .map_err(|e| MatchmakingServiceError::RepositoryError(e.to_string()))?;

        Ok(user)
    }

    pub async fn leave_queue(
        &self,
        player: &User,
        queue_type: &str,
    ) -> Result<(), MatchmakingServiceError> {
        let player_id = &player.id;

        if player_id.is_empty() {
            return Err(MatchmakingServiceError::ValidationError(
                "Player ID cannot be empty".to_string(),
            ));
        }

        if queue_type.is_empty() {
            return Err(MatchmakingServiceError::ValidationError(
                "Queue type cannot be empty".to_string(),
            ));
        }

        self.repository
            .leave_queue(player_id, queue_type, player.rating)
            .await
            .map_err(|e| MatchmakingServiceError::RepositoryError(e.to_string()))
    }
}
