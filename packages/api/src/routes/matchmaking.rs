use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

use crate::{middleware::auth::AuthenticatedUser, state::AppState};
use shared::models::matchmaking::{requests::JoinQueueRequest, responses::ErrorResponse};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/matchmaking/join", post(join_queue))
        .route("/matchmaking/leave", post(leave_queue))
}

async fn join_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(payload): Json<JoinQueueRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let player_id = authenticated_user.user_id.to_string();

    // Get the user to access their rating
    let user = match state.user_service.get_user_by_id(&player_id).await {
        Ok(user) => user,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get user: {}", e),
            };
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    match state
        .matchmaking_service
        .join_queue(&user, &payload.queue_type)
        .await
    {
        Ok(_matchmaking_user) => Ok(StatusCode::OK),
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            match e {
                crate::services::errors::matchmaking_service_errors::MatchmakingServiceError::ValidationError(_) => {
                    Err((StatusCode::BAD_REQUEST, Json(error_response)))
                }
                crate::services::errors::matchmaking_service_errors::MatchmakingServiceError::RepositoryError(_) => {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
                _ => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))),
            }
        }
    }
}

async fn leave_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(payload): Json<JoinQueueRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let player_id = authenticated_user.user_id.to_string();

    // Get the user to access their rating
    let user = match state.user_service.get_user_by_id(&player_id).await {
        Ok(user) => user,
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get user: {}", e),
            };
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    match state
        .matchmaking_service
        .leave_queue(&user, &payload.queue_type)
        .await
    {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            match e {
                crate::services::errors::matchmaking_service_errors::MatchmakingServiceError::ValidationError(_) => {
                    Err((StatusCode::BAD_REQUEST, Json(error_response)))
                }
                crate::services::errors::matchmaking_service_errors::MatchmakingServiceError::RepositoryError(_) => {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
                _ => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))),
            }
        }
    }
}
