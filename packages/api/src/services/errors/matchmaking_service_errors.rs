use std::fmt;

#[derive(Debug)]
pub enum MatchmakingServiceError {
    RepositoryError(String),
    UserAlreadyExists,
    UserNotFound,
    ValidationError(String),
    SerializationError(String),
}

impl fmt::Display for MatchmakingServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatchmakingServiceError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            MatchmakingServiceError::UserAlreadyExists => write!(f, "User already exists"),
            MatchmakingServiceError::UserNotFound => write!(f, "User not found"),
            MatchmakingServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            MatchmakingServiceError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for MatchmakingServiceError {}
