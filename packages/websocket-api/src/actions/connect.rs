use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_connect(
    user_id: &str,
    connection_id: &str,
    state: AppState,
) -> Result<Value, Error> {
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
