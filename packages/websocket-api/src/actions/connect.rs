use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};
use shared::services::auth_service::AuthServiceTrait;

use crate::state::AppState;

pub async fn handle_connect(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    let connection_id = event.request_context.connection_id.as_deref().unwrap_or("");

    // Extract token from query parameters
    let token = event
        .query_string_parameters
        .first("token")
        .ok_or_else(|| Error::from("Missing token query parameter"))?;

    // Authenticate using JWT token
    let player_id = match state.auth_service.extract_user_id_from_token(token) {
        Ok(user_id) => user_id,
        Err(e) => {
            return Ok(json!({
                "statusCode": 401,
                "body": json!({"error": format!("Authentication failed: {:?}", e)}).to_string()
            }));
        }
    };

    // Store the authenticated connection
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
        "statusCode": 200,
        "body": json!({"message": "Connected successfully", "player_id": player_id}).to_string()
    }))
}
