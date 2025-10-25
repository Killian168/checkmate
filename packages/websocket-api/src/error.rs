use shared::services::errors::auth_service_errors::AuthServiceError;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    AuthService(AuthServiceError),
    WebSocketError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::AuthService(err) => write!(f, "Auth service error: {}", err),
            ApiError::WebSocketError(msg) => write!(f, "WebSocket error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<AuthServiceError> for ApiError {
    fn from(error: AuthServiceError) -> Self {
        ApiError::AuthService(error)
    }
}
