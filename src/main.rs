use axum::{routing::get, Router};
use lambda_http::{run, tracing, Error};
use std::env::set_var;

mod routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    // Set up services
    // let config = aws_config::load_from_env().await;
    // let client = aws_sdk_dynamodb::Client::new(&config);

    // let activities_repository = Arc::new(DynamoDbActivityRepository::new(client.clone()));
    // let activities_service = Arc::new(ActivitiesService::new(activities_repository));

    // let user_repository = Arc::new(DynamoDbUserRepository::new(client.clone()));
    // let user_service = Arc::new(UserService::new(user_repository));

    // let auth_service = Arc::new(AuthService::new(user_service.clone()));

    // let app_state = models::AppState {
    //     activities_service,
    //     user_service,
    //     auth_service,
    // };

    // Merge routes
    let app = Router::new().route("/health", get(routes::health::health_check));
    // .merge(routes::auth::routes())
    // .merge(routes::activities::routes())
    // .with_state(app_state);

    run(app).await
}
