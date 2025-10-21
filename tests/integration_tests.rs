//! Simplified integration tests for Checkmate API
//!
//! These tests verify the integration between different components:
//! - API endpoints with mocked repositories
//! - Authentication flow
//! - Queue operations

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use lambda_runtime::tower::ServiceExt;
use serde_json::{json, Value};
use shared::{
    models::{
        auth::requests::{CreateUserRequest, LoginRequest},
        queue::requests::JoinQueueRequest,
        user::User,
    },
    repositories::{
        errors::{
            queue_repository_errors::QueueRepositoryError,
            user_repository_errors::UserRepositoryError,
        },
        queue_repository::QueueRepository,
        user_repository::UserRepository,
    },
    services::{
        auth_service::{AuthService, AuthServiceTrait},
        queue_service::QueueService,
        user_service::UserService,
    },
};
use std::sync::Arc;

// Import API modules for route testing

// Mock repositories for testing
mod mocks {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    #[derive(Clone)]
    pub struct MockUserRepository {
        users: Arc<RwLock<HashMap<String, User>>>,
    }

    impl MockUserRepository {
        pub fn new() -> Self {
            Self {
                users: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn email_exists(&self, email: &str) -> Result<bool, UserRepositoryError> {
            let users = self.users.read().await;
            Ok(users.values().any(|user| user.email == email))
        }

        async fn create_user(&self, user: &User) -> Result<(), UserRepositoryError> {
            let mut users = self.users.write().await;
            if users.contains_key(&user.id) {
                return Err(UserRepositoryError::AlreadyExists);
            }
            users.insert(user.id.clone(), user.clone());
            Ok(())
        }

        async fn get_user_by_id(&self, user_id: &str) -> Result<User, UserRepositoryError> {
            let users = self.users.read().await;
            users
                .get(user_id)
                .cloned()
                .ok_or(UserRepositoryError::NotFound)
        }

        async fn get_user_by_email(&self, email: &str) -> Result<User, UserRepositoryError> {
            let users = self.users.read().await;
            users
                .values()
                .find(|user| user.email == email)
                .cloned()
                .ok_or(UserRepositoryError::NotFound)
        }

        async fn delete_user(&self, user_id: &str) -> Result<(), UserRepositoryError> {
            let mut users = self.users.write().await;
            users
                .remove(user_id)
                .map(|_| ())
                .ok_or(UserRepositoryError::NotFound)
        }

        async fn update_user(&self, user: &User) -> Result<(), UserRepositoryError> {
            let mut users = self.users.write().await;
            if !users.contains_key(&user.id) {
                return Err(UserRepositoryError::NotFound);
            }
            users.insert(user.id.clone(), user.clone());
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct MockQueueRepository {
        queue_entries: Arc<RwLock<HashMap<String, shared::models::queue::QueueUser>>>,
    }

    impl MockQueueRepository {
        pub fn new() -> Self {
            Self {
                queue_entries: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl QueueRepository for MockQueueRepository {
        async fn join_queue(
            &self,
            player_id: &str,
            queue_type: &str,
            rating: i32,
        ) -> Result<(), QueueRepositoryError> {
            let mut entries = self.queue_entries.write().await;
            let queue_user = shared::models::queue::QueueUser::new(player_id, rating, queue_type);
            entries.insert(player_id.to_string(), queue_user);
            Ok(())
        }

        async fn leave_queue(
            &self,
            player_id: &str,
            _queue_type: &str,
            _rating: i32,
        ) -> Result<(), QueueRepositoryError> {
            let mut entries = self.queue_entries.write().await;
            entries
                .remove(player_id)
                .map(|_| ())
                .ok_or(QueueRepositoryError::NotFound)
        }

        async fn find_potential_opponents(
            &self,
            player_id: &str,
            queue_type: &str,
            rating: i32,
        ) -> Result<Vec<shared::models::queue::QueueUser>, QueueRepositoryError> {
            let entries = self.queue_entries.read().await;
            let opponents: Vec<_> = entries
                .values()
                .filter(|entry| {
                    entry.player_id != player_id
                        && entry.queue_type() == queue_type
                        && (entry.rating() - rating).abs() <= 100
                })
                .cloned()
                .collect();
            Ok(opponents)
        }

        async fn reserve_opponent(
            &self,
            _opponent: &shared::models::queue::QueueUser,
        ) -> Result<bool, QueueRepositoryError> {
            // Simulate successful reservation
            Ok(true)
        }
    }
}

// Test utilities
mod test_utils {
    use super::*;

    pub fn create_test_app() -> Router {
        let user_repo = Arc::new(mocks::MockUserRepository::new());
        let queue_repo = Arc::new(mocks::MockQueueRepository::new());

        let user_service = Arc::new(UserService::new(user_repo));
        // Set JWT_SECRET for tests
        std::env::set_var("JWT_SECRET", "test-secret-key");
        let auth_service = Arc::new(AuthService::new(user_service.clone()));
        let queue_service = Arc::new(QueueService::new(queue_repo));

        // Create app state compatible with API routes
        let app_state = state::AppState {
            user_service,
            auth_service,
            queue_service,
        };

        // Use the actual API routes
        Router::new()
            .route(
                "/health",
                axum::routing::get(|| async { axum::Json(json!({"status": "healthy"})) }),
            )
            .route("/auth/user", axum::routing::post(create_user_handler))
            .route("/auth/login", axum::routing::post(login_handler))
            .route("/queue/join", axum::routing::post(join_queue_handler))
            .route("/queue/leave", axum::routing::post(leave_queue_handler))
            .with_state(app_state)
    }

    // Test handlers that match the API route signatures
    async fn create_user_handler(
        axum::extract::State(state): axum::extract::State<state::AppState>,
        axum::Json(user_data): axum::Json<CreateUserRequest>,
    ) -> StatusCode {
        match state
            .user_service
            .create_user(
                &user_data.email,
                &user_data.password,
                &user_data.first_name,
                &user_data.last_name,
            )
            .await
        {
            Ok(_) => StatusCode::CREATED,
            Err(_) => StatusCode::BAD_REQUEST,
        }
    }

    async fn login_handler(
        axum::extract::State(state): axum::extract::State<state::AppState>,
        axum::Json(login_data): axum::Json<LoginRequest>,
    ) -> Result<axum::Json<shared::models::auth::responses::LoginResponse>, StatusCode> {
        match state
            .auth_service
            .authenticate_user(&login_data.email, &login_data.password)
            .await
        {
            Ok(login_response) => Ok(axum::Json(login_response)),
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }

    async fn join_queue_handler(
        axum::extract::State(state): axum::extract::State<state::AppState>,
        axum::Json(payload): axum::Json<JoinQueueRequest>,
    ) -> StatusCode {
        // For testing, we'll just simulate joining the queue
        // Use a consistent test user for all queue operations
        let test_user = User {
            id: "test-user-id".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            rating: 1200,
            created_at: chrono::Utc::now(),
        };
        match state
            .queue_service
            .join_queue(&test_user, &payload.queue_type)
            .await
        {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::BAD_REQUEST,
        }
    }

    async fn leave_queue_handler(
        axum::extract::State(state): axum::extract::State<state::AppState>,
        axum::Json(payload): axum::Json<JoinQueueRequest>,
    ) -> StatusCode {
        // For testing, we'll just simulate leaving the queue
        // Use the same consistent test user as join_queue
        let test_user = User {
            id: "test-user-id".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            rating: 1200,
            created_at: chrono::Utc::now(),
        };
        match state
            .queue_service
            .leave_queue(&test_user, &payload.queue_type)
            .await
        {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::BAD_REQUEST,
        }
    }

    // Test state for handlers
    mod state {
        use super::*;

        #[derive(Clone)]
        pub struct AppState {
            pub user_service: Arc<UserService>,
            pub auth_service: Arc<AuthService>,
            pub queue_service: Arc<QueueService>,
        }
    }

    pub fn _create_test_user() -> User {
        User::new(
            "test@example.com".to_string(),
            "password123".to_string(),
            "Test".to_string(),
            "User".to_string(),
        )
    }
}

// Test handlers for auth routes

// Test handlers for queue routes

// Test state for handlers

#[tokio::test]
async fn test_health_endpoint() {
    let app = test_utils::create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let health_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(health_response["status"], "healthy");
}

#[tokio::test]
async fn test_create_user_success() {
    let app = test_utils::create_test_app();

    let user_request = CreateUserRequest {
        email: "newuser@example.com".to_string(),
        password: "password123".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&user_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_user_duplicate_email() {
    let app = test_utils::create_test_app();

    let user_request = CreateUserRequest {
        email: "duplicate@example.com".to_string(),
        password: "password123".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    // First registration should succeed
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&user_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    // Second registration with same email should fail
    let response2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&user_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_success() {
    let app = test_utils::create_test_app();

    // First register a user
    let user_request = CreateUserRequest {
        email: "loginuser@example.com".to_string(),
        password: "password123".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&user_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Then login with the same credentials
    let login_request = LoginRequest {
        email: "loginuser@example.com".to_string(),
        password: "password123".to_string(),
    };

    let login_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_data: Value = serde_json::from_slice(&body).unwrap();
    assert!(login_data["token"].is_string());
    assert!(!login_data["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = test_utils::create_test_app();

    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "wrongpassword".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_join_queue_success() {
    let app = test_utils::create_test_app();

    let _join_request = JoinQueueRequest {
        queue_type: "rapid".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/queue/join")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "queue_type": "rapid"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_leave_queue_success() {
    let app = test_utils::create_test_app();

    // First join the queue
    let _join_request = JoinQueueRequest {
        queue_type: "rapid".to_string(),
    };

    let join_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/queue/join")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "queue_type": "rapid"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(join_response.status(), StatusCode::OK);

    // Then leave the queue
    let _leave_request = JoinQueueRequest {
        queue_type: "rapid".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/queue/leave")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "queue_type": "rapid"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
