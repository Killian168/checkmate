use aws_config::BehaviorVersion;
use aws_lambda_events::event::cognito::CognitoEventUserPoolsPostConfirmation;
use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_runtime::{
    run, service_fn,
    tracing::{info, init_default_subscriber},
    Error, LambdaEvent,
};
use serde_json;

use shared::User;
use tokio::sync::OnceCell as AsyncOnceCell;
lazy_static::lazy_static! {
    static ref TABLE_NAME: String = std::env::var("USERS_TABLE")
        .expect("USERS_TABLE environment variable not set");
}

static DYNAMO_CLIENT: AsyncOnceCell<DynamoClient> = AsyncOnceCell::const_new();

async fn get_dynamo_client() -> Result<&'static DynamoClient, Error> {
    DYNAMO_CLIENT
        .get_or_init(|| async {
            let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
            DynamoClient::new(&config)
        })
        .await;
    Ok(DYNAMO_CLIENT.get().unwrap())
}

async fn function_handler(
    event: LambdaEvent<CognitoEventUserPoolsPostConfirmation>,
) -> Result<serde_json::Value, Error> {
    let event_data = event.payload;

    // Extract user_id from sub
    let user_id = event_data
        .request
        .user_attributes
        .get("sub")
        .ok_or("Missing sub in user attributes")?
        .clone();

    let dynamo_client = get_dynamo_client().await?;

    // Prepare user struct
    let user = User {
        user_id,
        rating: 1200,
    };

    // Serialize to DynamoDB item
    let item = serde_dynamo::to_item(user).map_err(|e| format!("Serialization error: {:?}", e))?;

    dynamo_client
        .put_item()
        .table_name(TABLE_NAME.as_str())
        .set_item(Some(item))
        .send()
        .await
        .map_err(|e| format!("Failed to put item: {:?}", e))?;

    info!(
        "Successfully created user profile for {}",
        event_data
            .clone()
            .cognito_event_user_pools_header
            .user_name
            .unwrap_or_else(|| "unknown".to_string())
    );

    Ok(serde_json::to_value(event_data)?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_default_subscriber();

    run(service_fn(function_handler)).await
}
