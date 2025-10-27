use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{from_str, json, Value};
use tracing::{debug, error};

use crate::state::AppState;

use shared::models::queue::requests::JoinQueueRequest;

pub async fn handle_join_queue(
    event: &ApiGatewayWebsocketProxyRequest,
    user_id: &str,
    state: AppState,
) -> Result<Value, Error> {
    let payload: JoinQueueRequest = match event.body.as_ref() {
        Some(body) => {
            from_str(body).map_err(|e| Error::from(format!("Invalid join queue request: {}", e)))?
        }
        None => {
            return Ok(json!({
                "statusCode": 400,
                "body": json!({"error": "Missing join queue request body"}).to_string()
            }))
        }
    };

    let user = state
        .user_service
        .get_user_by_id(user_id)
        .await
        .map_err(|e| {
            error!("Failed to get user {}: {}", user_id, e);
            Error::from(e)
        })?;

    state
        .queue_service
        .join_queue(&user, &payload.queue_type)
        .await
        .map_err(|e| {
            error!("Failed to join queue for user {}: {}", user_id, e);
            Error::from(e)
        })?;

    debug!("User {} joined queue: {}", user_id, payload.queue_type);
    Ok(json!({
        "statusCode": 200,
        "body": json!({"message": "Joined queue successfully"}).to_string()
    }))
}
