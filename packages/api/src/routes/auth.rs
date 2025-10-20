use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use lambda_http::tracing::{debug, error, warn};

use crate::{middleware::auth::AuthenticatedUser, state::AppState};
use shared::models::auth::requests::{CreateUserRequest, LoginRequest};
use shared::models::auth::responses::LoginResponse;
use shared::services::errors::auth_service_errors::AuthServiceError;
use shared::services::errors::user_service_errors::UserServiceError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/user", post(create_user))
        .route("/auth/user", get(get_user))
        .route("/auth/user", delete(delete_user))
        .route("/auth/login", post(login))
}

async fn create_user(
    State(state): State<AppState>,
    Json(user_data): Json<CreateUserRequest>,
) -> Result<StatusCode, StatusCode> {
    match state
        .user_service
        .create_user(
            &user_data.email,
            &user_data.password,
            &user_data.first_name,
            &user_data.last_name,
        )
        .await
    {
        Ok(_) => {
            debug!("User created successfully: {}", user_data.email);
            Ok(StatusCode::CREATED)
        }
        Err(e) => {
            error!("Failed to create user {}: {}", user_data.email, e);
            Err(match e {
                UserServiceError::UserAlreadyExists => {
                    warn!("User already exists");
                    StatusCode::CONFLICT
                }
                UserServiceError::UserNotFound => {
                    warn!("User not found");
                    StatusCode::NOT_FOUND
                }
                UserServiceError::ValidationError(msg) => {
                    warn!("Validation error: {}", msg);
                    StatusCode::BAD_REQUEST
                }
                UserServiceError::RepositoryError(error_details) => {
                    error!("DynamoDB error: {}", error_details);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                UserServiceError::SerializationError(error_details) => {
                    error!("Serialization error: {}", error_details);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            })
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(login_data): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    match state
        .auth_service
        .authenticate_user(&login_data.email, &login_data.password)
        .await
    {
        Ok(login_response) => Ok(Json(login_response)),
        Err(e) => {
            error!("Failed to authenticate user {}: {}", login_data.email, e);
            Err(match e {
                AuthServiceError::InvalidCredentials => {
                    warn!("Invalid credentials");
                    StatusCode::UNAUTHORIZED
                }
                AuthServiceError::ValidationError(msg) => {
                    warn!("Validation error: {}", msg);
                    StatusCode::BAD_REQUEST
                }
                AuthServiceError::UserServiceError(user_service_error) => {
                    error!("User service error: {}", user_service_error);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                AuthServiceError::JwtError(error_details) => {
                    error!("JWT error: {}", error_details);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                AuthServiceError::InvalidToken | AuthServiceError::ExpiredToken => {
                    error!("Token error");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            })
        }
    }
}

async fn get_user(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<shared::models::user::User>, StatusCode> {
    match state
        .user_service
        .get_user_by_id(&authenticated_user.user_id)
        .await
    {
        Ok(user) => {
            debug!(
                "User retrieved successfully: {}",
                authenticated_user.user_id
            );
            Ok(Json(user))
        }
        Err(e) => {
            error!(
                "Failed to retrieve user {}: {}",
                authenticated_user.user_id, e
            );
            Err(match e {
                UserServiceError::UserNotFound => {
                    warn!("User not found");
                    StatusCode::NOT_FOUND
                }
                UserServiceError::ValidationError(msg) => {
                    warn!("Validation error: {}", msg);
                    StatusCode::BAD_REQUEST
                }
                UserServiceError::UserAlreadyExists => {
                    warn!("User already exists");
                    StatusCode::CONFLICT
                }
                UserServiceError::RepositoryError(error_details) => {
                    error!("DynamoDB error: {}", error_details);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                UserServiceError::SerializationError(error_details) => {
                    error!("Serialization error: {}", error_details);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            })
        }
    }
}

async fn delete_user(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<StatusCode, StatusCode> {
    match state
        .user_service
        .delete_user(&authenticated_user.user_id)
        .await
    {
        Ok(_) => {
            debug!("User deleted successfully: {}", authenticated_user.user_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!(
                "Failed to delete user {}: {}",
                authenticated_user.user_id, e
            );
            Err(match e {
                UserServiceError::UserNotFound => {
                    warn!("User not found");
                    StatusCode::NOT_FOUND
                }
                UserServiceError::ValidationError(msg) => {
                    warn!("Validation error: {}", msg);
                    StatusCode::BAD_REQUEST
                }
                UserServiceError::UserAlreadyExists => {
                    warn!("User already exists");
                    StatusCode::CONFLICT
                }
                UserServiceError::RepositoryError(error_details) => {
                    error!("DynamoDB error: {}", error_details);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                UserServiceError::SerializationError(error_details) => {
                    error!("Serialization error: {}", error_details);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            })
        }
    }
}
