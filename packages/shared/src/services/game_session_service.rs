use std::sync::Arc;

use crate::{
    models::{game_session::GameSession, move_request::MoveRequest, queue::QueueUser},
    repositories::game_repository::GameSessionRepository,
    services::{
        chess_service::ChessService, errors::game_session_service_errors::GameSessionServiceError,
    },
};

#[derive(Clone)]
pub struct GameSessionService {
    repository: Arc<dyn GameSessionRepository + Send + Sync>,
    chess_service: Arc<ChessService>,
}

impl GameSessionService {
    pub fn new(repository: Arc<dyn GameSessionRepository + Send + Sync>) -> Self {
        GameSessionService {
            repository,
            chess_service: Arc::new(ChessService::new()),
        }
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

    pub async fn make_move(
        &self,
        move_request: &MoveRequest,
        player_id: &str,
    ) -> Result<GameSession, GameSessionServiceError> {
        // Get the game session
        let mut game_session = self
            .get_game_session(&move_request.game_session_id)
            .await?
            .ok_or_else(|| {
                GameSessionServiceError::ValidationError("Game session not found".to_string())
            })?;

        // Validate and make the move using ChessService
        self.chess_service
            .validate_and_make_move(&mut game_session, move_request, player_id)?;

        // Update the game session in the repository
        self.repository.update_game_session(&game_session).await?;

        Ok(game_session)
    }
}
