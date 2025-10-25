use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_disconnect(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    let connection_id = event.request_context.connection_id.as_deref().unwrap_or("");
    if let Err(_e) = state
        .websocket_service
        .remove_connection_by_id(connection_id)
        .await
    {}

    Ok(json!({
        "statusCode": 200
    }))
}
