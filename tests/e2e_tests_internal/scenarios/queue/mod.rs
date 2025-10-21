use reqwest::StatusCode;
use tokio::task::JoinHandle;

use crate::e2e_tests_internal::client::TestClient;
use crate::e2e_tests_internal::config::E2EConfig;
use crate::e2e_tests_internal::utils::{TestResult, TestTimer, TestUser};

/// Test multiple users interacting with the queue system
pub async fn test_multiple_users_queue_interaction() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let user_count = 3;

    let mut handles: Vec<JoinHandle<TestResult<()>>> = Vec::new();

    for _i in 0..user_count {
        let base_url = config.base_url.clone();
        let handle = tokio::spawn(async move {
            let mut client = TestClient::new(base_url);
            let test_user = TestUser::new("Multi", "User");

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

            // Join queue
            let join_status = client.join_queue("rapid").await?;
            assert_eq!(join_status, StatusCode::OK);

            // Leave queue
            let leave_status = client.leave_queue("rapid").await?;
            assert_eq!(leave_status, StatusCode::OK);

            // Delete account
            let delete_status = client.delete_account().await?;
            assert_eq!(delete_status, StatusCode::NO_CONTENT);

            Ok(())
        });

        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        result??; // Unwrap both JoinError and TestResult
    }

    timer.assert_within_timeout(config.timeout);
    Ok(())
}

/// Test concurrent queue operations
pub async fn test_concurrent_queue_operations() -> TestResult<()> {
    let config = E2EConfig::default();
    let timer = TestTimer::start();
    let user_count = 3; // Reduced for stability

    let mut handles: Vec<JoinHandle<TestResult<()>>> = Vec::new();

    for _i in 0..user_count {
        let base_url = config.base_url.clone();
        let handle = tokio::spawn(async move {
            let mut client = TestClient::new(base_url);
            let test_user = TestUser::new("Concurrent", "User");

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

            // Join queue
            let join_status = client.join_queue("rapid").await?;
            assert_eq!(join_status, StatusCode::OK);

            // Leave queue
            let leave_status = client.leave_queue("rapid").await?;
            assert_eq!(leave_status, StatusCode::OK);

            // Delete account
            let delete_status = client.delete_account().await?;
            assert_eq!(delete_status, StatusCode::NO_CONTENT);

            Ok(())
        });

        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        result??;
    }

    timer.assert_within_timeout(config.timeout);
    Ok(())
}
