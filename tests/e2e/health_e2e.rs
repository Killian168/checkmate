use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

/// Get the API base URL from environment variable or use default dev endpoint
fn get_api_url() -> String {
    env::var("API_URL").unwrap_or_else(|_| {
        // If not set, try to read from serverless output or fail with helpful message
        panic!(
            "API_URL environment variable not set. \
             Please set it to your deployed API endpoint. \
             Example: export API_URL=https://abc123.execute-api.eu-west-1.amazonaws.com"
        )
    })
}

#[tokio::test]
async fn e2e_health_endpoint_returns_200() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request to health endpoint");

    assert_eq!(
        response.status(),
        200,
        "Health endpoint should return 200 OK"
    );
}

#[tokio::test]
async fn e2e_health_endpoint_returns_json() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request to health endpoint");

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header should be present");

    assert!(
        content_type
            .to_str()
            .unwrap()
            .contains("application/json"),
        "Content-Type should be application/json"
    );
}

#[tokio::test]
async fn e2e_health_endpoint_returns_correct_body() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request to health endpoint");

    assert_eq!(response.status(), 200);

    let health: HealthResponse = response
        .json()
        .await
        .expect("Response should be valid JSON matching HealthResponse schema");

    assert_eq!(health.status, "healthy");
    assert_eq!(health.service, "checkmate");
    assert!(!health.version.is_empty(), "Version should not be empty");
}

#[tokio::test]
async fn e2e_health_endpoint_handles_invalid_methods() {
    let api_url = get_api_url();
    let client = Client::new();

    // Test POST
    let response = client
        .post(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send POST request");

    assert_ne!(
        response.status(),
        200,
        "POST should not be allowed on health endpoint"
    );

    // Test PUT
    let response = client
        .put(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_ne!(
        response.status(),
        200,
        "PUT should not be allowed on health endpoint"
    );

    // Test DELETE
    let response = client
        .delete(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_ne!(
        response.status(),
        200,
        "DELETE should not be allowed on health endpoint"
    );
}

#[tokio::test]
async fn e2e_health_endpoint_handles_query_parameters() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/health?foo=bar&baz=qux", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request with query parameters");

    assert_eq!(
        response.status(),
        200,
        "Health endpoint should ignore query parameters and return 200"
    );

    let health: HealthResponse = response
        .json()
        .await
        .expect("Response should be valid JSON");

    assert_eq!(health.status, "healthy");
}

#[tokio::test]
async fn e2e_health_endpoint_response_time() {
    let api_url = get_api_url();
    let client = Client::new();

    let start = std::time::Instant::now();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request");

    let duration = start.elapsed();

    assert_eq!(response.status(), 200);

    // Health check should respond quickly (within 5 seconds, accounting for cold starts)
    assert!(
        duration < Duration::from_secs(5),
        "Health check took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn e2e_health_endpoint_concurrent_requests() {
    let api_url = get_api_url();
    let client = Client::new();

    let mut handles = Vec::new();

    // Send 10 concurrent requests
    for _ in 0..10 {
        let client = client.clone();
        let url = api_url.clone();

        let handle = tokio::spawn(async move {
            client
                .get(format!("{}/health", url))
                .timeout(Duration::from_secs(30))
                .send()
                .await
                .expect("Failed to send concurrent request")
                .status()
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let status = handle.await.expect("Task panicked");
        assert_eq!(status, 200, "All concurrent requests should succeed");
    }
}

#[tokio::test]
async fn e2e_health_endpoint_consistency() {
    let api_url = get_api_url();
    let client = Client::new();

    let mut responses = Vec::new();

    // Make 5 sequential requests
    for _ in 0..5 {
        let response = client
            .get(format!("{}/health", api_url))
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .expect("Failed to send request");

        let health: HealthResponse = response.json().await.expect("Failed to parse JSON");
        responses.push(health);
    }

    // All responses should be identical
    let first = &responses[0];
    for response in &responses[1..] {
        assert_eq!(
            response, first,
            "All health check responses should be consistent"
        );
    }
}

#[tokio::test]
async fn e2e_health_endpoint_with_custom_headers() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/health", api_url))
        .header("User-Agent", "E2E-Test/1.0")
        .header("X-Custom-Header", "test-value")
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request with custom headers");

    assert_eq!(
        response.status(),
        200,
        "Health endpoint should accept requests with custom headers"
    );
}

#[tokio::test]
async fn e2e_health_endpoint_cold_start() {
    let api_url = get_api_url();
    let client = Client::new();

    // First request might be a cold start
    let start = std::time::Instant::now();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request");

    let cold_start_duration = start.elapsed();

    assert_eq!(response.status(), 200);

    println!("Cold start duration: {:?}", cold_start_duration);

    // Subsequent request should be faster (warm)
    let start = std::time::Instant::now();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send second request");

    let warm_duration = start.elapsed();

    assert_eq!(response.status(), 200);

    println!("Warm duration: {:?}", warm_duration);

    // Warm request should generally be faster than cold start
    // (but we won't assert this as it's not guaranteed)
}

#[tokio::test]
async fn e2e_health_endpoint_schema_validation() {
    let api_url = get_api_url();
    let client = Client::new();

    let response = client
        .get(format!("{}/health", api_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to send request");

    let json: serde_json::Value = response
        .json()
        .await
        .expect("Response should be valid JSON");

    // Verify all required fields exist
    assert!(json.get("status").is_some(), "status field is required");
    assert!(json.get("service").is_some(), "service field is required");
    assert!(json.get("version").is_some(), "version field is required");

    // Verify field types
    assert!(
        json["status"].is_string(),
        "status should be a string"
    );
    assert!(
        json["service"].is_string(),
        "service should be a string"
    );
    assert!(
        json["version"].is_string(),
        "version should be a string"
    );

    // Verify field values
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "checkmate");
}

#[tokio::test]
async fn e2e_health_endpoint_https() {
    let api_url = get_api_url();

    assert!(
        api_url.starts_with("https://"),
        "API should be served over HTTPS for security"
    );
}

#[tokio::test]
async fn e2e_health_endpoint_availability() {
    let api_url = get_api_url();
    let client = Client::new();

    let iterations = 20;
    let mut success_count = 0;

    for _ in 0..iterations {
        let response = client
            .get(format!("{}/health", api_url))
            .timeout(Duration::from_secs(30))
            .send()
            .await;

        if let Ok(resp) = response {
            if resp.status() == 200 {
                success_count += 1;
            }
        }

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Should have high availability (at least 95%)
    let availability = (success_count as f64 / iterations as f64) * 100.0;
    assert!(
        availability >= 95.0,
        "Availability should be at least 95%, got {:.2}%",
        availability
    );
}
