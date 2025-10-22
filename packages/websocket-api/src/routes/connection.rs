use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::state::AppState;

pub async fn handle_connect(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    let connection_id = event.request_context.connection_id.as_deref().unwrap_or("");
    info!("WebSocket connection established: {}", connection_id);
    debug!("Attempting to store connection in DynamoDB");

    // Extract player_id from query parameters if available
    let player_id = if let Some(player_id) = event.query_string_parameters.first("player_id") {
        debug!("Found player_id from query parameters: {}", player_id);
        player_id.to_string()
    } else {
        debug!("No player_id in query parameters, using connection_id as player_id");
        format!("player_{}", connection_id)
    };

    debug!("Calling websocket_service.store_connection");
    if let Err(e) = state
        .websocket_service
        .store_connection(&player_id, connection_id)
        .await
    {
        error!("Failed to store connection {}: {}", connection_id, e);
        debug!("Error details: {:?}", e);
        return Ok(json!({
            "statusCode": 500,
            "body": json!({"error": "Failed to store connection"}).to_string()
        }));
    }

    debug!("Successfully stored connection, returning 200 response");
    Ok(json!({
        "statusCode": 200
    }))
}

pub async fn handle_disconnect(connection_id: &str, state: AppState) -> Result<Value, Error> {
    info!("WebSocket connection disconnected: {}", connection_id);
    debug!("Attempting to remove connection from DynamoDB");

    debug!("Calling websocket_service.remove_connection_by_id");
    if let Err(e) = state
        .websocket_service
        .remove_connection_by_id(connection_id)
        .await
    {
        error!("Failed to remove connection {}: {}", connection_id, e);
        debug!("Error details: {:?}", e);
    }

    Ok(json!({
        "statusCode": 200
    }))
}
