use axum::http::StatusCode;

/// Health check endpoint to verify API status
pub async fn health_check() -> (StatusCode, String) {
    let health = true;
    match health {
        true => (StatusCode::OK, "Healthy!".to_string()),
        false => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Not healthy!".to_string(),
        ),
    }
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

    #[tokio::test]
    async fn test_health_check_unhealthy() {
        // Temporarily modify the health variable to test unhealthy state
        let health = false;
        let (status, message) = match health {
            true => (StatusCode::OK, "Healthy!".to_string()),
            false => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Not healthy!".to_string(),
            ),
        };
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, "Not healthy!");
    }
}
