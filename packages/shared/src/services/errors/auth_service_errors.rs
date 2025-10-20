use crate::services::errors::user_service_errors::UserServiceError;
use std::fmt;

#[derive(Debug)]
pub enum AuthServiceError {
    UserServiceError(UserServiceError),
    InvalidCredentials,
    JwtError(String),
    ValidationError(String),
    InvalidToken,
    ExpiredToken,
}

impl fmt::Display for AuthServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthServiceError::UserServiceError(err) => write!(f, "User service error: {}", err),
            AuthServiceError::InvalidCredentials => write!(f, "Invalid email or password"),
            AuthServiceError::JwtError(msg) => write!(f, "JWT error: {}", msg),
            AuthServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AuthServiceError::InvalidToken => write!(f, "Invalid JWT token"),
            AuthServiceError::ExpiredToken => write!(f, "JWT token has expired"),
        }
    }
}

impl std::error::Error for AuthServiceError {}
