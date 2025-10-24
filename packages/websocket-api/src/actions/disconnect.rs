use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_disconnect(connection_id: &str, state: AppState) -> Result<Value, Error> {
    if let Err(_e) = state
        .websocket_service
        .remove_connection_by_id(connection_id)
        .await
    {}

    Ok(json!({
        "statusCode": 200
    }))
}
