use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use tracing_subscriber;

use shared::models::user::User;
use shared::repositories::user_repository::DynamoDbUserRepository;
use shared::services::user_service::UserService;

#[derive(Deserialize)]
struct CognitoEvent {
    request: Request,
}

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "userAttributes")]
    user_attributes: UserAttributes,
}

#[derive(Deserialize)]
struct UserAttributes {
    sub: String,
    email: String,
    #[serde(rename = "given_name")]
    given_name: Option<String>,
    #[serde(rename = "family_name")]
    family_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    run(service_fn(user_signup_handler)).await
}

async fn user_signup_handler(event: LambdaEvent<CognitoEvent>) -> Result<(), Error> {
    let user_attrs = &event.payload.request.user_attributes;

    let user_id = user_attrs.sub.clone();
    let email = user_attrs.email.clone();
    let first_name = user_attrs
        .given_name
        .clone()
        .unwrap_or_else(|| "".to_string());
    let last_name = user_attrs
        .family_name
        .clone()
        .unwrap_or_else(|| "".to_string());

    info!("Creating user: {}", user_id);

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let user_repo = DynamoDbUserRepository::new(client);
    let user_service = UserService::new(Arc::new(user_repo));

    let user = User::new(user_id.clone(), email, first_name, last_name);
    user_service
        .create_user(&user)
        .await
        .map_err(|e| Error::from(format!("Failed to create user {}: {}", user_id, e)))?;

    info!("User created successfully: {}", user_id);
    Ok(())
}
