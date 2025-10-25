use crate::repositories::errors::game_repository_errors::GameSessionRepositoryError;
use crate::services::errors::chess_service_errors::ChessServiceError;

#[derive(Debug)]
pub enum GameSessionServiceError {
    RepositoryError(GameSessionRepositoryError),
    ValidationError(String),
    ChessError(ChessServiceError),
}

impl std::fmt::Display for GameSessionServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameSessionServiceError::RepositoryError(err) => {
                write!(f, "Repository error: {}", err)
            }
            GameSessionServiceError::ValidationError(msg) => {
                write!(f, "Validation error: {}", msg)
            }
            GameSessionServiceError::ChessError(err) => {
                write!(f, "Chess error: {}", err)
            }
        }
    }
}

impl std::error::Error for GameSessionServiceError {}

impl From<GameSessionRepositoryError> for GameSessionServiceError {
    fn from(err: GameSessionRepositoryError) -> Self {
        GameSessionServiceError::RepositoryError(err)
    }
}

impl From<ChessServiceError> for GameSessionServiceError {
    fn from(err: ChessServiceError) -> Self {
        GameSessionServiceError::ChessError(err)
    }
}
