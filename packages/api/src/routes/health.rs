use axum::http::StatusCode;

/// Health check endpoint to verify API status
pub async fn health_check() -> (StatusCode, String) {
    return (StatusCode::OK, "Healthy!".to_string());
}
