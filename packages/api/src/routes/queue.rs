use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use lambda_http::tracing::{debug, error};

use crate::{error::ApiError, middleware::auth::AuthenticatedUser, state::AppState};
use shared::models::queue::requests::JoinQueueRequest;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/queue/join", post(join_queue))
        .route("/queue/leave", post(leave_queue))
}

async fn join_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(payload): Json<JoinQueueRequest>,
) -> Result<StatusCode, ApiError> {
    let player_id = authenticated_user.user_id.clone();

    // Get the user to access their rating
    let user = state
        .user_service
        .get_user_by_id(&player_id)
        .await
        .map_err(|e| {
            error!("Failed to get user {}: {}", player_id, e);
            ApiError::from(e)
        })?;

    state
        .queue_service
        .join_queue(&user, &payload.queue_type)
        .await
        .map_err(|e| {
            error!("Failed to join queue for user {}: {}", player_id, e);
            ApiError::from(e)
        })?;

    debug!("User {} joined queue: {}", player_id, payload.queue_type);
    Ok(StatusCode::OK)
}

async fn leave_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(payload): Json<JoinQueueRequest>,
) -> Result<StatusCode, ApiError> {
    let player_id = authenticated_user.user_id.to_string();

    // Get the user to access their rating
    let user = state
        .user_service
        .get_user_by_id(&player_id)
        .await
        .map_err(|e| {
            error!("Failed to get user {}: {}", player_id, e);
            ApiError::from(e)
        })?;

    state
        .queue_service
        .leave_queue(&user, &payload.queue_type)
        .await
        .map_err(|e| {
            error!("Failed to leave queue for user {}: {}", player_id, e);
            ApiError::from(e)
        })?;

    debug!("User {} left queue: {}", player_id, payload.queue_type);
    Ok(StatusCode::OK)
}
