use lambda_http::{Body, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WebSocketEvent {
    pub request_context: RequestContext,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RequestContext {
    pub connection_id: String,
    pub route_key: String,
    pub event_type: String,
    pub domain_name: String,
    pub stage: String,
}

#[derive(Debug, Serialize)]
pub struct WebSocketResponse {
    pub status_code: u16,
    pub body: Option<String>,
}

pub async fn websocket_handler(
    event: Request,
    state: AppState,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    // Parse the event body
    let body = event.body();
    let body_str = std::str::from_utf8(body)?;
    let websocket_event: WebSocketEvent = serde_json::from_str(body_str)?;
    let route_key = &websocket_event.request_context.route_key;
    let connection_id = &websocket_event.request_context.connection_id;

    match route_key.as_str() {
        "$connect" => handle_connect(connection_id, state).await,
        "$disconnect" => handle_disconnect(connection_id, state).await,
        "$default" => handle_default_message(&websocket_event, state).await,
        _ => {
            error!("Unknown route key: {}", route_key);
            Ok(Response::builder()
                .status(400)
                .body(Body::from(json!({"error": "Unknown route"}).to_string()))?)
        }
    }
}

async fn handle_connect(
    connection_id: &str,
    _state: AppState,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    info!("WebSocket connection established: {}", connection_id);

    // For now, we'll accept all connections
    // In a real implementation, you might validate tokens or other auth here

    Ok(Response::builder().status(200).body(Body::Empty)?)
}

async fn handle_disconnect(
    connection_id: &str,
    state: AppState,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    info!("WebSocket connection disconnected: {}", connection_id);

    // Remove connection from DynamoDB
    if let Err(e) = state
        .websocket_service
        .remove_connection_by_id(connection_id)
        .await
    {
        error!("Failed to remove connection {}: {}", connection_id, e);
    }

    Ok(Response::builder().status(200).body(Body::Empty)?)
}

async fn handle_default_message(
    event: &WebSocketEvent,
    _state: AppState,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let connection_id = &event.request_context.connection_id;

    if let Some(body) = &event.body {
        info!(
            "Received message from connection {}: {}",
            connection_id, body
        );

        // Parse the message
        let message: Value = serde_json::from_str(body)?;

        // Handle different message types
        if let Some(action) = message.get("action").and_then(|a| a.as_str()) {
            match action {
                "ping" => {
                    let response = json!({
                        "action": "pong",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    return Ok(Response::builder()
                        .status(200)
                        .body(Body::from(response.to_string()))?);
                }
                "get_connection_status" => {
                    // Try to find the player ID for this connection
                    // For now, we'll just return basic connection info
                    let response = json!({
                        "action": "connection_status",
                        "status": "connected",
                        "connection_id": connection_id,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    return Ok(Response::builder()
                        .status(200)
                        .body(Body::from(response.to_string()))?);
                }
                "echo" => {
                    if let Some(data) = message.get("data") {
                        let response = json!({
                            "action": "echo",
                            "data": data,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });

                        return Ok(Response::builder()
                            .status(200)
                            .body(Body::from(response.to_string()))?);
                    }
                }
                _ => {
                    // Unknown action
                    let response = json!({
                        "action": "error",
                        "message": "Unknown action",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    return Ok(Response::builder()
                        .status(400)
                        .body(Body::from(response.to_string()))?);
                }
            }
        }

        // If no action specified, return error
        let response = json!({
            "action": "error",
            "message": "No action specified",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(Response::builder()
            .status(400)
            .body(Body::from(response.to_string()))?)
    } else {
        // No body in message
        let response = json!({
            "action": "error",
            "message": "No message body",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(Response::builder()
            .status(400)
            .body(Body::from(response.to_string()))?)
    }
}
