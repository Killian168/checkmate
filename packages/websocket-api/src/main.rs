use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env::set_var;
use std::sync::Arc;
use tracing::{debug, error, info};

pub mod state;

use shared::repositories::websocket_repository::DynamoDbWebSocketRepository;
use shared::services::websocket_service::WebSocketService;

#[derive(Debug, Deserialize)]
pub struct WebSocketEvent {
    #[serde(rename = "requestContext")]
    pub request_context: RequestContext,
    pub body: Option<String>,
    #[serde(rename = "queryStringParameters")]
    pub query_string_parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RequestContext {
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    #[serde(rename = "routeKey")]
    pub route_key: String,
    #[serde(rename = "eventType")]
    pub event_type: String,
    #[serde(rename = "domainName")]
    pub domain_name: String,
    pub stage: String,
}

#[derive(Debug, Serialize)]
pub struct WebSocketResponse {
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    pub body: Option<String>,
    #[serde(rename = "headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
}

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

    run(service_fn(|event: LambdaEvent<WebSocketEvent>| {
        websocket_handler(event, app_state.clone())
    }))
    .await
}

async fn websocket_handler(
    event: LambdaEvent<WebSocketEvent>,
    state: state::AppState,
) -> Result<WebSocketResponse, Error> {
    debug!("Received WebSocket event: {:?}", event);

    let websocket_event = event.payload;
    let route_key = &websocket_event.request_context.route_key;
    let connection_id = &websocket_event.request_context.connection_id;

    debug!(
        "Processing route_key: {}, connection_id: {}",
        route_key, connection_id
    );

    match route_key.as_str() {
        "$connect" => {
            debug!("Handling $connect route");
            handle_connect(connection_id, &websocket_event, state).await
        }
        "$disconnect" => {
            debug!("Handling $disconnect route");
            handle_disconnect(connection_id, state).await
        }
        "$default" => {
            debug!("Handling $default route");
            handle_default_message(&websocket_event, state).await
        }
        _ => {
            error!("Unknown route key: {}", route_key);
            Ok(WebSocketResponse {
                status_code: 400,
                body: Some(json!({"error": "Unknown route"}).to_string()),
                headers: None,
            })
        }
    }
}

async fn handle_connect(
    connection_id: &str,
    event: &WebSocketEvent,
    state: state::AppState,
) -> Result<WebSocketResponse, Error> {
    info!("WebSocket connection established: {}", connection_id);
    debug!("Attempting to store connection in DynamoDB");

    // Extract player_id from query parameters if available
    let player_id = if let Some(query_params) = &event.query_string_parameters {
        if let Some(player_id) = query_params.get("player_id").and_then(|p| p.as_str()) {
            debug!("Found player_id from query parameters: {}", player_id);
            player_id.to_string()
        } else {
            debug!("No player_id in query parameters, using connection_id as player_id");
            format!("player_{}", connection_id)
        }
    } else {
        debug!("No query parameters, using connection_id as player_id");
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
        return Ok(WebSocketResponse {
            status_code: 500,
            body: Some(json!({"error": "Failed to store connection"}).to_string()),
            headers: None,
        });
    }

    debug!("Successfully stored connection, returning 200 response");
    Ok(WebSocketResponse {
        status_code: 200,
        body: None,
        headers: None,
    })
}

async fn handle_disconnect(
    connection_id: &str,
    state: state::AppState,
) -> Result<WebSocketResponse, Error> {
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

    Ok(WebSocketResponse {
        status_code: 200,
        body: None,
        headers: None,
    })
}

async fn handle_default_message(
    event: &WebSocketEvent,
    state: state::AppState,
) -> Result<WebSocketResponse, Error> {
    let connection_id = &event.request_context.connection_id;
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
                return Ok(WebSocketResponse {
                    status_code: 400,
                    body: Some(
                        json!({
                            "action": "error",
                            "message": "Invalid JSON format",
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        })
                        .to_string(),
                    ),
                    headers: None,
                });
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
                        return Ok(WebSocketResponse {
                            status_code: 500,
                            body: Some(json!({"error": "Failed to send response"}).to_string()),
                            headers: None,
                        });
                    }

                    return Ok(WebSocketResponse {
                        status_code: 200,
                        body: None,
                        headers: None,
                    });
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
                        return Ok(WebSocketResponse {
                            status_code: 500,
                            body: Some(json!({"error": "Failed to send response"}).to_string()),
                            headers: None,
                        });
                    }

                    return Ok(WebSocketResponse {
                        status_code: 200,
                        body: None,
                        headers: None,
                    });
                }
                "echo" => {
                    debug!("Handling echo action");
                    if let Some(data) = message.get("data") {
                        debug!("Echo data: {:?}", data);
                        let response = json!({
                            "action": "echo",
                            "data": data,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });

                        // Send response via API Gateway Management API
                        if let Err(e) = state
                            .websocket_service
                            .send_message(connection_id, &response.to_string())
                            .await
                        {
                            error!("Failed to send echo response: {}", e);
                            return Ok(WebSocketResponse {
                                status_code: 500,
                                body: Some(json!({"error": "Failed to send response"}).to_string()),
                                headers: None,
                            });
                        }

                        return Ok(WebSocketResponse {
                            status_code: 200,
                            body: None,
                            headers: None,
                        });
                    }
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
                        return Ok(WebSocketResponse {
                            status_code: 500,
                            body: Some(json!({"error": "Failed to send response"}).to_string()),
                            headers: None,
                        });
                    }

                    return Ok(WebSocketResponse {
                        status_code: 200,
                        body: None,
                        headers: None,
                    });
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
            return Ok(WebSocketResponse {
                status_code: 500,
                body: Some(json!({"error": "Failed to send response"}).to_string()),
                headers: None,
            });
        }

        Ok(WebSocketResponse {
            status_code: 200,
            body: None,
            headers: None,
        })
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
            return Ok(WebSocketResponse {
                status_code: 500,
                body: Some(json!({"error": "Failed to send response"}).to_string()),
                headers: None,
            });
        }

        Ok(WebSocketResponse {
            status_code: 200,
            body: None,
            headers: None,
        })
    }
}
