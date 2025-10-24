use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_default_message(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    let connection_id = event.request_context.connection_id.as_deref().unwrap_or("");

    let response = json!({
        "action": "error",
        "message": "Unknown action",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if let Err(_e) = state
        .websocket_service
        .send_message(connection_id, &response.to_string())
        .await
    {
        return Ok(json!({
            "statusCode": 500,
            "body": json!({"error": "Failed to send response"}).to_string()
        }));
    }

    Ok(json!({
        "statusCode": 200
    }))
}
