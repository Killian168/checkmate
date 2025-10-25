use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_connect(
    _event: &ApiGatewayWebsocketProxyRequest,
    _state: AppState,
) -> Result<Value, Error> {
    // Temporarily bypass all authentication and storage for testing
    // TODO: Restore proper WebSocket connection logic

    Ok(json!({
        "statusCode": 200,
        "body": json!({"message": "Connected successfully"}).to_string()
    }))
}
