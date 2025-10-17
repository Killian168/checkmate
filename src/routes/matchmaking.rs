use axum::{
    extract::State,
    http::StatusCode,
    routing::post, Router,
};

use crate::{middleware::auth::AuthenticatedUser, models::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/matchmaking/join", post(join_queue))
        .route("/matchmaking/leave", post(leave_queue))
}

async fn join_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> (StatusCode, String) {
    return (StatusCode::OK, "Healthy!".to_string());
}

async fn leave_queue(
    State(state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> (StatusCode, String) {
    return (StatusCode::OK, "Healthy!".to_string());
}
