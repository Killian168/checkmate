use std::fmt;

#[derive(Debug)]
pub enum UserServiceError {
    RepositoryError(String),
    UserAlreadyExists,
    UserNotFound,
    ValidationError(String),
    SerializationError(String),
}

impl fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserServiceError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            UserServiceError::UserAlreadyExists => write!(f, "User already exists"),
            UserServiceError::UserNotFound => write!(f, "User not found"),
            UserServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            UserServiceError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for UserServiceError {}
