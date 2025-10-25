use std::sync::Arc;

use crate::{
    models::{game_session::GameSession, queue::QueueUser},
    repositories::game_repository::GameSessionRepository,
    services::errors::game_session_service_errors::GameSessionServiceError,
};

#[derive(Clone)]
pub struct GameSessionService {
    repository: Arc<dyn GameSessionRepository + Send + Sync>,
}

impl GameSessionService {
    pub fn new(repository: Arc<dyn GameSessionRepository + Send + Sync>) -> Self {
        GameSessionService { repository }
    }

    pub async fn create_game_session(
        &self,
        player_1: &QueueUser,
        player_2: &QueueUser,
    ) -> Result<(), GameSessionServiceError> {
        let game_session = GameSession::new(&player_1.player_id, &player_2.player_id);
        self.repository.create_game_session(&game_session).await?;
        Ok(())
    }

    pub async fn get_game_session(
        &self,
        session_id: &str,
    ) -> Result<Option<GameSession>, GameSessionServiceError> {
        self.repository
            .get_game_session(session_id)
            .await
            .map_err(GameSessionServiceError::from)
    }
}
