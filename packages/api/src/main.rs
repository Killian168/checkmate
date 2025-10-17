use axum::{routing::get, Router};
use lambda_http::{run, tracing, Error};
use std::env::set_var;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub mod middleware;
pub mod routes;
pub mod services;
pub mod state;

use services::auth_service::AuthService;
use services::matchmaking_service::MatchmakingService;
use services::user_service::UserService;
use shared::repositories::matchmaking_repository::DynamoDbMatchmakingUserRepository;
use shared::repositories::user_repository::DynamoDbUserRepository;

#[tokio::main]
async fn main() -> Result<(), Error> {
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    // Set up services
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let user_repository = Arc::new(DynamoDbUserRepository::new(client.clone()));
    let user_service = Arc::new(UserService::new(user_repository));
    let auth_service = Arc::new(AuthService::new(user_service.clone()));

    let matchmaking_repository = Arc::new(DynamoDbMatchmakingUserRepository::new(client.clone()));
    let matchmaking_service = Arc::new(MatchmakingService::new(matchmaking_repository));

    let app_state = state::AppState {
        user_service,
        auth_service,
        matchmaking_service,
    };

    // Configure CORS
    // ToDo: Tighten this up
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Merge routes
    let app = Router::new()
        .route("/health", get(routes::health::health_check))
        .merge(routes::auth::routes())
        .merge(routes::matchmaking::routes())
        .layer(cors)
        .with_state(app_state);

    run(app).await
}
