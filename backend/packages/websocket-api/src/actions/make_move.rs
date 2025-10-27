use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use chrono;
use lambda_runtime::Error;
use serde_json::{from_str, json, Value};

use crate::state::AppState;

use shared::models::move_request::MoveRequest;

pub async fn handle_make_move(
    event: &ApiGatewayWebsocketProxyRequest,
    user_id: &str,
    state: AppState,
) -> Result<Value, Error> {
    // Parse the move request from the event body
    let move_request: MoveRequest = match event.body.as_ref() {
        Some(body) => {
            from_str(body).map_err(|e| Error::from(format!("Invalid move request: {}", e)))?
        }
        None => {
            return Ok(json!({
                "statusCode": 400,
                "body": json!({"error": "Missing move request body"}).to_string()
            }))
        }
    };

    // Make the move
    let updated_session = state
        .game_session_service
        .make_move(&move_request, user_id)
        .await
        .map_err(|e| Error::from(format!("Failed to make move: {}", e)))?;

    // Determine the opponent
    let opponent_id = if updated_session.player1_id == user_id {
        &updated_session.player2_id
    } else {
        &updated_session.player1_id
    };

    // Notify the opponent
    if let Ok(Some(opponent_connection_id)) =
        state.websocket_service.get_connection_id(opponent_id).await
    {
        let notification = json!({
            "action": "move_made",
            "game_session_id": updated_session.session_id,
            "from_square": move_request.from_square,
            "to_square": move_request.to_square,
            "promotion_piece": move_request.promotion_piece,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        let _ = state
            .websocket_service
            .send_message(&opponent_connection_id, &notification.to_string())
            .await;
    }

    Ok(json!({
        "statusCode": 200,
        "body": json!({"message": "Move made successfully", "game_session": updated_session}).to_string()
    }))
}
