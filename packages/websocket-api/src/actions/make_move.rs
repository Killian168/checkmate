use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_make_move(
    _event: &ApiGatewayWebsocketProxyRequest,
    _connection_id: &str,
    _state: AppState,
) -> Result<Value, Error> {
    // Temporarily bypass all complex logic for testing WebSocket infrastructure
    // TODO: Restore proper make_move logic

    // Temporarily return success to test WebSocket infrastructure
    Ok(json!({
        "statusCode": 200,
        "body": json!({"message": "Move handler working"}).to_string()
    }))
}
