use lambda_runtime::Error;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handle_default_message(connection_id: &str, state: AppState) -> Result<Value, Error> {
    let response = json!({
        "action": "error",
        "message": "Unknown action",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    state
        .websocket_service
        .send_message(connection_id, &response.to_string())
        .await
        .map_err(Error::from)?;

    Ok(json!({
        "statusCode": 200
    }))
}
