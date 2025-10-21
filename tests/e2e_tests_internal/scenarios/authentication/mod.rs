use reqwest::StatusCode;

use crate::e2e_tests_internal::client::TestClient;
use crate::e2e_tests_internal::config::E2EConfig;
use crate::e2e_tests_internal::utils::{verify_health_endpoint, TestResult, TestTimer, TestUser};

/// Test complete user journey from registration to deletion
pub async fn test_complete_user_journey() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();

    // Verify health endpoint first
    let health_ok = verify_health_endpoint(&config.base_url).await?;
    assert!(health_ok, "Health endpoint should be healthy");

    let mut client = TestClient::from_config(&config);

    // Generate unique test credentials
    let test_user = TestUser::new("Complete", "Journey");

    // Step 1: Register user
    let register_status = client
        .register_user(
            &test_user.email,
            &test_user.password,
            &test_user.first_name,
            &test_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    // Step 2: Login
    let token = client.login(&test_user.email, &test_user.password).await?;
    assert!(!token.is_empty());
    assert!(client.is_authenticated());

    // Step 3: Get user info
    let user_info = client.get_user_info().await?;
    assert_eq!(user_info["email"], test_user.email);
    assert_eq!(user_info["first_name"], test_user.first_name);
    assert_eq!(user_info["last_name"], test_user.last_name);

    // Step 4: Join queue
    let join_status = client.join_queue("rapid").await?;
    assert_eq!(join_status, StatusCode::OK);

    // Step 5: Leave queue
    let leave_status = client.leave_queue("rapid").await?;
    assert_eq!(leave_status, StatusCode::OK);

    // Step 6: Delete account
    let delete_status = client.delete_account().await?;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test authentication flow including unauthorized access attempts
pub async fn test_authentication_flow() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let mut client = TestClient::from_config(&config);

    let test_user = TestUser::new("Auth", "Test");

    // Test: Cannot access protected endpoints without authentication
    let join_result = client.join_queue("rapid").await;
    assert!(
        join_result.is_err(),
        "Should not be able to join queue without auth"
    );

    let user_info_result = client.get_user_info().await;
    assert!(
        user_info_result.is_err(),
        "Should not be able to get user info without auth"
    );

    let delete_result = client.delete_account().await;
    assert!(
        delete_result.is_err(),
        "Should not be able to delete account without auth"
    );

    // Register and login
    let register_status = client
        .register_user(
            &test_user.email,
            &test_user.password,
            &test_user.first_name,
            &test_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    let token = client.login(&test_user.email, &test_user.password).await?;
    assert!(!token.is_empty());
    assert!(client.is_authenticated());

    // Now should be able to access protected endpoints
    let user_info = client.get_user_info().await?;
    assert_eq!(user_info["email"], test_user.email);

    // Clean up
    let delete_status = client.delete_account().await?;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test various error scenarios in authentication
pub async fn test_error_scenarios() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let mut client = TestClient::from_config(&config);

    // Test: Login with non-existent user
    let non_existent_login = client.login("nonexistent@example.com", "password").await;
    assert!(
        non_existent_login.is_err(),
        "Should not be able to login with non-existent user"
    );

    // Register a valid user first
    let test_user = TestUser::new("Error", "Test");

    let register_status = client
        .register_user(
            &test_user.email,
            &test_user.password,
            &test_user.first_name,
            &test_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    // Test: Login with wrong password
    let wrong_password_login = client.login(&test_user.email, "wrongpassword").await;
    assert!(
        wrong_password_login.is_err(),
        "Should not be able to login with wrong password"
    );

    // Test: Join queue with invalid token
    let mut invalid_client = TestClient::from_config(&config);
    invalid_client.auth_token = Some("invalid-token".to_string());
    let invalid_join = invalid_client.join_queue("rapid").await;
    assert!(
        invalid_join.is_err(),
        "Should not be able to join queue with invalid token"
    );

    // Clean up valid user
    let login_result = client.login(&test_user.email, &test_user.password).await;
    if login_result.is_ok() {
        let delete_status = client.delete_account().await?;
        assert_eq!(delete_status, StatusCode::NO_CONTENT);
    }

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test session persistence across client instances
pub async fn test_session_persistence() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let mut client = TestClient::from_config(&config);

    let test_user = TestUser::new("Session", "Test");

    // Register user
    let register_status = client
        .register_user(
            &test_user.email,
            &test_user.password,
            &test_user.first_name,
            &test_user.last_name,
        )
        .await?;
    assert_eq!(register_status, StatusCode::CREATED);

    // Login and verify session
    let token = client.login(&test_user.email, &test_user.password).await?;
    assert!(!token.is_empty());

    let user_info1 = client.get_user_info().await?;
    assert_eq!(user_info1["email"], test_user.email);

    // Create new client instance with same credentials
    let mut new_client = TestClient::from_config(&config);
    let new_token = new_client
        .login(&test_user.email, &test_user.password)
        .await?;
    assert!(!new_token.is_empty());

    let user_info2 = new_client.get_user_info().await?;
    assert_eq!(user_info2["email"], test_user.email);
    assert_eq!(user_info2["id"], user_info1["id"]);

    // Clean up
    let delete_status = new_client.delete_account().await?;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    timer.assert_within_timeout(config.timeout);
    Ok(())
}
