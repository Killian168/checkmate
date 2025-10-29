use aws_config::BehaviorVersion;
use aws_sdk_cognitoidentityprovider::Client as CognitoClient;
use aws_sdk_dynamodb::Client as DynamoClient;
use axum::{
    routing::{delete, get},
    Extension, Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub mod auth;
pub mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub dynamo_client: DynamoClient,
    pub cognito_client: CognitoClient,
    pub users_table: String,
    pub cognito_user_pool_id: String,
}

impl AppState {
    pub async fn new() -> Self {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let dynamo_client = DynamoClient::new(&config);
        let cognito_client = CognitoClient::new(&config);
        let users_table = std::env::var("USERS_TABLE").expect("USERS_TABLE must be set");
        let cognito_user_pool_id =
            std::env::var("COGNITO_USER_POOL_ID").expect("COGNITO_USER_POOL_ID must be set");

        Self {
            dynamo_client,
            cognito_client,
            users_table,
            cognito_user_pool_id,
        }
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/users/me", get(handlers::users::get_me))
        .route("/users/me", delete(handlers::users::delete_me))
        .layer(Extension(state))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
}
