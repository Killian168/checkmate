use std::fmt;

#[derive(Debug)]
pub enum QueueServiceError {
    RepositoryError(String),
    UserAlreadyExists,
    UserNotFound,
    ValidationError(String),
    SerializationError(String),
}

impl fmt::Display for QueueServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueueServiceError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            QueueServiceError::UserAlreadyExists => write!(f, "User already exists"),
            QueueServiceError::UserNotFound => write!(f, "User not found"),
            QueueServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            QueueServiceError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for QueueServiceError {}
