use axum::http::StatusCode;

/// Health check endpoint to verify API status
pub async fn health_check() -> (StatusCode, String) {
    return (StatusCode::OK, "Healthy!".to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_healthy() {
        let (status, message) = health_check().await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(message, "Healthy!");
    }
}
