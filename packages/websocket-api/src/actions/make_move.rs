use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_make_move(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    Ok(json!({
        "statusCode": 200
    }))
}
