use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

#[tracing::instrument]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "checkmate".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_json() {
        let response = health_check().await;
        assert_eq!(response.0.status, "healthy");
    }

    #[tokio::test]
    async fn test_health_check_service_name() {
        let response = health_check().await;
        assert_eq!(response.0.service, "checkmate");
    }

    #[tokio::test]
    async fn test_health_check_version_is_set() {
        let response = health_check().await;
        assert!(!response.0.version.is_empty());
        assert_eq!(response.0.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_health_response_structure() {
        let response = health_check().await;
        let health = response.0;

        assert_eq!(health.status, "healthy");
        assert_eq!(health.service, "checkmate");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            service: "checkmate".to_string(),
            version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"service\":\"checkmate\""));
        assert!(json.contains("\"version\":\"0.1.0\""));
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{"status":"healthy","service":"checkmate","version":"0.1.0"}"#;
        let response: HealthResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, "healthy");
        assert_eq!(response.service, "checkmate");
        assert_eq!(response.version, "0.1.0");
    }

    #[test]
    fn test_health_response_clone() {
        let response1 = HealthResponse {
            status: "healthy".to_string(),
            service: "checkmate".to_string(),
            version: "0.1.0".to_string(),
        };

        let response2 = response1.clone();
        assert_eq!(response1, response2);
    }

    #[test]
    fn test_health_response_debug() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            service: "checkmate".to_string(),
            version: "0.1.0".to_string(),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("healthy"));
        assert!(debug_str.contains("checkmate"));
        assert!(debug_str.contains("0.1.0"));
    }
}
