use crate::repositories::errors::game_repository_errors::GameSessionRepositoryError;

#[derive(Debug)]
pub enum GameSessionServiceError {
    RepositoryError(GameSessionRepositoryError),
    ValidationError(String),
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
        }
    }
}

impl std::error::Error for GameSessionServiceError {}

impl From<GameSessionRepositoryError> for GameSessionServiceError {
    fn from(err: GameSessionRepositoryError) -> Self {
        GameSessionServiceError::RepositoryError(err)
    }
}
