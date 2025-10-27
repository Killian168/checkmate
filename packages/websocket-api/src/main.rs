use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::env::set_var;
use std::sync::Arc;

pub mod actions;
pub mod error;
pub mod state;

use shared::repositories::websocket_repository::DynamoDbWebSocketRepository;

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
        user_service,
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

    // Extract Cognito authorizer claims
    let authorizer = websocket_event
        .request_context
        .authorizer
        .as_ref()
        .ok_or_else(|| Error::from("No authorizer"))?;
    let claims = authorizer
        .get("claims")
        .and_then(|c| c.as_object())
        .ok_or_else(|| Error::from("No claims"))?;
    let user_id = claims
        .get("sub")
        .and_then(|s| s.as_str())
        .ok_or_else(|| Error::from("No sub"))?
        .to_string();

    // Route request
    let route_key = websocket_event
        .request_context
        .route_key
        .as_deref()
        .unwrap_or("");

    match route_key {
        "$connect" => handle_connect(claims, connection_id, state).await,
        "$disconnect" => handle_disconnect(connection_id, state).await,
        "$default" => handle_default_message(connection_id, state).await,
        "make_move" => handle_make_move(&websocket_event, &user_id, state).await,
        _ => Ok(json!({
            "statusCode": 400,
            "body": json!({"error": "Unknown route"}).to_string()
        })),
    }
}
