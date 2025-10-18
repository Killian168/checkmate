use crate::repositories::matchmaking_repository::MatchmakingRepositoryError;

#[derive(Debug)]
pub enum MatchmakingServiceError {
    RepositoryError(String),
}

impl std::fmt::Display for MatchmakingServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchmakingServiceError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
        }
    }
}

impl std::error::Error for MatchmakingServiceError {}

impl From<MatchmakingRepositoryError> for MatchmakingServiceError {
    fn from(error: MatchmakingRepositoryError) -> Self {
        MatchmakingServiceError::RepositoryError(error.to_string())
    }
}
