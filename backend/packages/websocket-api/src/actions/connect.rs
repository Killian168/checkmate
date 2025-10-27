use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_connect(
    claims: &serde_json::Map<String, Value>,
    connection_id: &str,
    state: AppState,
) -> Result<Value, Error> {
    let user_id = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::from("No sub in claims"))?;

    state
        .websocket_service
        .store_connection(user_id, connection_id)
        .await
        .map_err(Error::from)?;

    Ok(json!({
        "statusCode": 200,
        "body": json!({"message": "Connected successfully"}).to_string()
    }))
}
