use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::state::AppState;

pub async fn handle_default_message(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    let connection_id = event.request_context.connection_id.as_deref().unwrap_or("");
    debug!(
        "Processing default message for connection: {}",
        connection_id
    );

    if let Some(body) = &event.body {
        info!(
            "Received message from connection {}: {}",
            connection_id, body
        );
        debug!("Raw message body: {:?}", body);

        // Parse the message
        debug!("Attempting to parse message as JSON");
        let message: Value = match serde_json::from_str(body) {
            Ok(msg) => {
                debug!("Successfully parsed message: {:?}", msg);
                msg
            }
            Err(e) => {
                error!("Failed to parse message: {}", e);
                debug!("Parse error details: {:?}", e);
                return Ok(json!({
                    "statusCode": 400,
                    "body": json!({
                        "action": "error",
                        "message": "Invalid JSON format",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }).to_string()
                }));
            }
        };

        // Handle different message types
        debug!("Looking for action field in message");
        if let Some(action) = message.get("action").and_then(|a| a.as_str()) {
            debug!("Found action: {}", action);
            match action {
                "ping" => {
                    debug!("Handling ping action");
                    let response = json!({
                        "action": "pong",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    // Send response via API Gateway Management API
                    if let Err(e) = state
                        .websocket_service
                        .send_message(connection_id, &response.to_string())
                        .await
                    {
                        error!("Failed to send pong response: {}", e);
                        return Ok(json!({
                            "statusCode": 500,
                            "body": json!({"error": "Failed to send response"}).to_string()
                        }));
                    }

                    return Ok(json!({
                        "statusCode": 200
                    }));
                }
                "get_connection_status" => {
                    debug!("Handling get_connection_status action");
                    let response = json!({
                        "action": "connection_status",
                        "status": "connected",
                        "connection_id": connection_id,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    // Send response via API Gateway Management API
                    if let Err(e) = state
                        .websocket_service
                        .send_message(connection_id, &response.to_string())
                        .await
                    {
                        error!("Failed to send connection status response: {}", e);
                        return Ok(json!({
                            "statusCode": 500,
                            "body": json!({"error": "Failed to send response"}).to_string()
                        }));
                    }

                    return Ok(json!({
                        "statusCode": 200
                    }));
                }
                _ => {
                    // Unknown action
                    debug!("Unknown action received: {}", action);
                    let response = json!({
                        "action": "error",
                        "message": "Unknown action",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    // Send response via API Gateway Management API
                    if let Err(e) = state
                        .websocket_service
                        .send_message(connection_id, &response.to_string())
                        .await
                    {
                        error!("Failed to send error response: {}", e);
                        return Ok(json!({
                            "statusCode": 500,
                            "body": json!({"error": "Failed to send response"}).to_string()
                        }));
                    }

                    return Ok(json!({
                        "statusCode": 200
                    }));
                }
            }
        }

        // If no action specified, return error
        debug!("No action specified in message");
        let response = json!({
            "action": "error",
            "message": "No action specified",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Send response via API Gateway Management API
        if let Err(e) = state
            .websocket_service
            .send_message(connection_id, &response.to_string())
            .await
        {
            error!("Failed to send error response: {}", e);
            return Ok(json!({
                "statusCode": 500,
                "body": json!({"error": "Failed to send response"}).to_string()
            }));
        }

        Ok(json!({
            "statusCode": 200
        }))
    } else {
        // No body in message
        debug!("No body found in WebSocket event");
        let response = json!({
            "action": "error",
            "message": "No message body",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Send response via API Gateway Management API
        if let Err(e) = state
            .websocket_service
            .send_message(connection_id, &response.to_string())
            .await
        {
            error!("Failed to send error response: {}", e);
            return Ok(json!({
                "statusCode": 500,
                "body": json!({"error": "Failed to send response"}).to_string()
            }));
        }

        Ok(json!({
            "statusCode": 200
        }))
    }
}
