use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use lambda_http::tracing::{debug, error};

use crate::{error::ApiError, middleware::auth::AuthenticatedUser, state::AppState};
use shared::models::auth::requests::{CreateUserRequest, LoginRequest};
use shared::models::auth::responses::LoginResponse;
use shared::services::auth_service::AuthServiceTrait;

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
) -> Result<StatusCode, ApiError> {
    state
        .user_service
        .create_user(
            &user_data.email,
            &user_data.password,
            &user_data.first_name,
            &user_data.last_name,
        )
        .await
        .map_err(|e| {
            error!("Failed to create user {}: {}", user_data.email, e);
            ApiError::from(e)
        })?;
    debug!("User created successfully: {}", user_data.email);
    Ok(StatusCode::CREATED)
}

async fn login(
    State(state): State<AppState>,
    Json(login_data): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    state
        .auth_service
        .authenticate_user(&login_data.email, &login_data.password)
        .await
        .map(Json)
        .map_err(|e| {
            error!("Failed to authenticate user {}: {}", login_data.email, e);
            ApiError::from(e)
        })
}

async fn get_user(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<shared::models::user::User>, ApiError> {
    state
        .user_service
        .get_user_by_id(&authenticated_user.user_id)
        .await
        .map(Json)
        .map_err(|e| {
            error!(
                "Failed to retrieve user {}: {}",
                authenticated_user.user_id, e
            );
            ApiError::from(e)
        })
}

async fn delete_user(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    state
        .user_service
        .delete_user(&authenticated_user.user_id)
        .await
        .map_err(|e| {
            error!(
                "Failed to delete user {}: {}",
                authenticated_user.user_id, e
            );
            ApiError::from(e)
        })?;
    debug!("User deleted successfully: {}", authenticated_user.user_id);
    Ok(StatusCode::NO_CONTENT)
}
