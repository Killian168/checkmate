use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use lambda_http::tracing::{debug, error};

use crate::{middleware::auth::AuthenticatedUser, state::AppState};
use shared::models::queue::requests::JoinQueueRequest;
use shared::services::errors::queue_service_errors::QueueServiceError;
use shared::services::errors::user_service_errors::UserServiceError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/queue/join", post(join_queue))
        .route("/queue/leave", post(leave_queue))
}

async fn join_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(payload): Json<JoinQueueRequest>,
) -> Result<StatusCode, StatusCode> {
    let player_id = authenticated_user.user_id.to_string();

    // Get the user to access their rating
    let user = match state.user_service.get_user_by_id(&player_id).await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to get user {}: {}", player_id, e);
            return Err(match e {
                UserServiceError::UserNotFound => StatusCode::NOT_FOUND,
                UserServiceError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserServiceError::UserAlreadyExists => StatusCode::CONFLICT,
                UserServiceError::RepositoryError(_) | UserServiceError::SerializationError(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            });
        }
    };

    match state
        .queue_service
        .join_queue(&user, &payload.queue_type)
        .await
    {
        Ok(_queue_user) => {
            debug!("User {} joined queue: {}", player_id, payload.queue_type);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to join queue for user {}: {}", player_id, e);
            Err(match e {
                QueueServiceError::ValidationError(_) => StatusCode::BAD_REQUEST,
                QueueServiceError::UserAlreadyExists => StatusCode::CONFLICT,
                QueueServiceError::UserNotFound => StatusCode::NOT_FOUND,
                QueueServiceError::RepositoryError(_)
                | QueueServiceError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

async fn leave_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(payload): Json<JoinQueueRequest>,
) -> Result<StatusCode, StatusCode> {
    let player_id = authenticated_user.user_id.to_string();

    // Get the user to access their rating
    let user = match state.user_service.get_user_by_id(&player_id).await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to get user {}: {}", player_id, e);
            return Err(match e {
                UserServiceError::UserNotFound => StatusCode::NOT_FOUND,
                UserServiceError::ValidationError(_) => StatusCode::BAD_REQUEST,
                UserServiceError::UserAlreadyExists => StatusCode::CONFLICT,
                UserServiceError::RepositoryError(_) | UserServiceError::SerializationError(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            });
        }
    };

    match state
        .queue_service
        .leave_queue(&user, &payload.queue_type)
        .await
    {
        Ok(()) => {
            debug!("User {} left queue: {}", player_id, payload.queue_type);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!("Failed to leave queue for user {}: {}", player_id, e);
            Err(match e {
                QueueServiceError::ValidationError(_) => StatusCode::BAD_REQUEST,
                QueueServiceError::UserAlreadyExists => StatusCode::CONFLICT,
                QueueServiceError::UserNotFound => StatusCode::NOT_FOUND,
                QueueServiceError::RepositoryError(_)
                | QueueServiceError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}
