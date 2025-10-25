use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::env::set_var;
use std::sync::Arc;

pub mod actions;
pub mod error;
pub mod state;

use crate::error::ApiError;
use shared::repositories::websocket_repository::DynamoDbWebSocketRepository;
use shared::services::auth_service::AuthServiceTrait;
use shared::services::errors::auth_service_errors::AuthServiceError;
use shared::services::game_session_service::GameSessionService;
use shared::services::websocket_service::WebSocketService;

use crate::actions::connect::handle_connect;
use crate::actions::default::handle_default_message;
use crate::actions::disconnect::handle_disconnect;
use crate::actions::make_move::handle_make_move;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Error> {
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Set up services with single config loading
    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    let user_repository = Arc::new(
        shared::repositories::user_repository::DynamoDbUserRepository::new(dynamodb_client.clone()),
    );
    let user_service = Arc::new(shared::services::user_service::UserService::new(
        user_repository,
    ));
    let auth_service = Arc::new(shared::services::auth_service::AuthService::new(
        user_service,
    ));

    let websocket_repository = Arc::new(DynamoDbWebSocketRepository::new(
        dynamodb_client.clone(),
        aws_sdk_apigatewaymanagement::Client::new(&config),
    ));
    let websocket_service = Arc::new(WebSocketService::new(websocket_repository));

    let game_session_repository = Arc::new(
        shared::repositories::game_repository::DynamoDbGameSessionRepository::new(
            dynamodb_client.clone(),
        ),
    );
    let game_session_service = Arc::new(GameSessionService::new(game_session_repository));

    let app_state = state::AppState {
        websocket_service,
        auth_service,
        game_session_service,
    };

    run(service_fn(
        |event: LambdaEvent<ApiGatewayWebsocketProxyRequest>| {
            websocket_handler(event, app_state.clone())
        },
    ))
    .await
}

async fn websocket_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    state: AppState,
) -> Result<Value, Error> {
    let websocket_event = event.payload;
    let connection_id = websocket_event
        .request_context
        .connection_id
        .as_ref()
        .ok_or_else(|| Error::from("Connection ID not found"))?;

    // Authenticate the user
    let user_id = match authenticate_user(&websocket_event, state.clone()).await {
        Ok(id) => id,
        Err(e) => {
            return Ok(json!({
                "statusCode": 401,
                "body": json!({"error": format!("{}", e)}).to_string()
            }));
        }
    };

    // Route request
    let route_key = websocket_event
        .request_context
        .route_key
        .as_deref()
        .unwrap_or("");

    match route_key {
        "$connect" => handle_connect(&user_id, connection_id, state).await,
        "$disconnect" => handle_disconnect(connection_id, state).await,
        "$default" => handle_default_message(connection_id, state).await,
        "make_move" => handle_make_move(&websocket_event, &user_id, state).await,
        _ => Ok(json!({
            "statusCode": 400,
            "body": json!({"error": "Unknown route"}).to_string()
        })),
    }
}

async fn authenticate_user(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<String, ApiError> {
    let auth_header = event
        .headers
        .get("Authorization")
        .ok_or_else(|| ApiError::AuthService(AuthServiceError::InvalidCredentials))?
        .to_str()
        .map_err(|_| {
            ApiError::AuthService(AuthServiceError::ValidationError(
                "Invalid header format".to_string(),
            ))
        })?;

    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::AuthService(AuthServiceError::InvalidCredentials));
    }

    // Extract the token (remove "Bearer " prefix)
    let token = &auth_header[7..];

    // Verify JWT and extract user ID
    let user_id = state
        .auth_service
        .extract_user_id_from_token(token)
        .map_err(|e| ApiError::from(e))?;

    Ok(user_id)
}
