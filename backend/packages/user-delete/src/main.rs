use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

use shared::repositories::user_repository::DynamoDbUserRepository;
use shared::services::user_service::UserService;

#[derive(Deserialize)]
struct EventBridgeEvent {
    detail: Detail,
}

#[derive(Deserialize)]
struct Detail {
    #[serde(rename = "userName")]
    user_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    run(service_fn(user_delete_handler)).await
}

async fn user_delete_handler(event: LambdaEvent<EventBridgeEvent>) -> Result<(), Error> {
    let user_id = event.payload.detail.user_name.clone();

    info!("Deleting user: {}", user_id);

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let user_repo = DynamoDbUserRepository::new(client);
    let user_service = UserService::new(Arc::new(user_repo));

    user_service
        .delete_user(&user_id)
        .await
        .map_err(|e| Error::from(format!("Failed to delete user {}: {}", user_id, e)))?;

    info!("User deleted successfully: {}", user_id);
    Ok(())
}
