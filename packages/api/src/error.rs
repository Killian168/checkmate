use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use shared::services::errors::{
    auth_service_errors::AuthServiceError, game_session_service_errors::GameSessionServiceError,
    queue_service_errors::QueueServiceError, user_service_errors::UserServiceError,
};

#[derive(Debug)]
pub enum ApiError {
    UserService(UserServiceError),
    AuthService(AuthServiceError),
    QueueService(QueueServiceError),
    GameSessionService(GameSessionServiceError),
    Unauthorized,
}

impl From<UserServiceError> for ApiError {
    fn from(error: UserServiceError) -> Self {
        ApiError::UserService(error)
    }
}

impl From<AuthServiceError> for ApiError {
    fn from(error: AuthServiceError) -> Self {
        ApiError::AuthService(error)
    }
}

impl From<QueueServiceError> for ApiError {
    fn from(error: QueueServiceError) -> Self {
        ApiError::QueueService(error)
    }
}

impl From<GameSessionServiceError> for ApiError {
    fn from(error: GameSessionServiceError) -> Self {
        ApiError::GameSessionService(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::UserService(UserServiceError::UserAlreadyExists) => StatusCode::CONFLICT,
            ApiError::UserService(UserServiceError::UserNotFound) => StatusCode::NOT_FOUND,
            ApiError::UserService(UserServiceError::ValidationError(_)) => StatusCode::BAD_REQUEST,
            ApiError::UserService(
                UserServiceError::RepositoryError(_) | UserServiceError::SerializationError(_),
            ) => StatusCode::INTERNAL_SERVER_ERROR,

            ApiError::AuthService(AuthServiceError::InvalidCredentials) => StatusCode::UNAUTHORIZED,
            ApiError::AuthService(AuthServiceError::ValidationError(_)) => StatusCode::BAD_REQUEST,
            ApiError::AuthService(AuthServiceError::UserServiceError(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::AuthService(AuthServiceError::JwtError(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::AuthService(
                AuthServiceError::InvalidToken | AuthServiceError::ExpiredToken,
            ) => StatusCode::UNAUTHORIZED,

            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,

            ApiError::QueueService(QueueServiceError::ValidationError(_)) => {
                StatusCode::BAD_REQUEST
            }
            ApiError::QueueService(QueueServiceError::UserAlreadyExists) => StatusCode::CONFLICT,
            ApiError::QueueService(QueueServiceError::UserNotFound) => StatusCode::NOT_FOUND,
            ApiError::QueueService(
                QueueServiceError::RepositoryError(_) | QueueServiceError::SerializationError(_),
            ) => StatusCode::INTERNAL_SERVER_ERROR,

            ApiError::GameSessionService(GameSessionServiceError::ValidationError(_)) => {
                StatusCode::BAD_REQUEST
            }
            ApiError::GameSessionService(
                GameSessionServiceError::RepositoryError(_)
                | GameSessionServiceError::ChessError(_),
            ) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        status.into_response()
    }
}
