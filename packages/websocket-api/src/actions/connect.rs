use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_connect(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    let connection_id = event.request_context.connection_id.as_deref().unwrap_or("");

    // Extract player_id from query parameters if available
    let player_id = if let Some(player_id) = event.query_string_parameters.first("player_id") {
        player_id.to_string()
    } else {
        format!("player_{}", connection_id)
    };

    if let Err(_e) = state
        .websocket_service
        .store_connection(&player_id, connection_id)
        .await
    {
        return Ok(json!({
            "statusCode": 500,
            "body": json!({"error": "Failed to store connection"}).to_string()
        }));
    }

    Ok(json!({
        "statusCode": 200
    }))
}
