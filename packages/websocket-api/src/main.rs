use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::env::set_var;
use std::sync::Arc;

pub mod actions;
pub mod state;

use shared::repositories::websocket_repository::DynamoDbWebSocketRepository;
use shared::services::websocket_service::WebSocketService;

use crate::actions::connect::handle_connect;
use crate::actions::default::handle_default_message;
use crate::actions::disconnect::handle_disconnect;
use crate::actions::make_move::handle_make_move;

#[tokio::main]
async fn main() -> Result<(), Error> {
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Set up services
    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);
    let api_gateway_client = aws_sdk_apigatewaymanagement::Client::new(&config);

    let websocket_repository = Arc::new(DynamoDbWebSocketRepository::new(
        dynamodb_client,
        api_gateway_client,
    ));
    let websocket_service = Arc::new(WebSocketService::new(websocket_repository));

    let app_state = state::AppState { websocket_service };

    run(service_fn(
        |event: LambdaEvent<ApiGatewayWebsocketProxyRequest>| {
            websocket_handler(event, app_state.clone())
        },
    ))
    .await
}

async fn websocket_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    state: state::AppState,
) -> Result<Value, Error> {
    let websocket_event = event.payload;
    let route_key = websocket_event
        .request_context
        .route_key
        .as_deref()
        .unwrap_or("");
    let connection_id = websocket_event
        .request_context
        .connection_id
        .as_deref()
        .unwrap_or("");

    match route_key {
        "$connect" => handle_connect(&websocket_event, state).await,
        "$disconnect" => handle_disconnect(connection_id, state).await,
        "$default" => handle_default_message(&websocket_event, state).await,
        "make_move" => handle_make_move(&websocket_event, state).await,
        _ => Ok(json!({
            "statusCode": 400,
            "body": json!({"error": "Unknown route"}).to_string()
        })),
    }
}
