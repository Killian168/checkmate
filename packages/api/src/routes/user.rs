use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use lambda_http::tracing::{debug, error};

use crate::{error::ApiError, middleware::auth::AuthenticatedUser, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user", get(get_user))
        .route("/user", delete(delete_user))
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
