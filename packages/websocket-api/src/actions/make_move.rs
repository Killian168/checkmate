use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::Error;
use serde_json::{json, Value};
use shared::models::move_request::MoveRequest;

use crate::state::AppState;

pub async fn handle_make_move(
    event: &ApiGatewayWebsocketProxyRequest,
    state: AppState,
) -> Result<Value, Error> {
    let connection_id = event.request_context.connection_id.as_deref().unwrap_or("");

    // Get authenticated player ID from connection
    let player_id = match state.websocket_service.get_player_id(connection_id).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return Ok(json!({
                "statusCode": 401,
                "body": json!({"error": "No authenticated connection found"}).to_string()
            }));
        }
        Err(e) => {
            return Ok(json!({
                "statusCode": 500,
                "body": json!({"error": format!("Failed to get player ID: {}", e)}).to_string()
            }));
        }
    };

    // Parse move request from message body
    let move_request: MoveRequest = match serde_json::from_str(
        &event.body.clone().unwrap_or_default(),
    ) {
        Ok(req) => req,
        Err(e) => {
            return Ok(json!({
                "statusCode": 400,
                "body": json!({"error": format!("Invalid move request format: {}", e)}).to_string()
            }));
        }
    };

    // Get the game session
    let game_session = match state
        .game_session_service
        .get_game_session(&move_request.game_session_id)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => {
            return Ok(json!({
                "statusCode": 404,
                "body": json!({"error": "Game session not found"}).to_string()
            }));
        }
        Err(e) => {
            return Ok(json!({
                "statusCode": 500,
                "body": json!({"error": format!("Failed to get game session: {}", e)}).to_string()
            }));
        }
    };

    // Validate that the player is part of this game
    if game_session.player1_id != player_id && game_session.player2_id != player_id {
        return Ok(json!({
            "statusCode": 403,
            "body": json!({"error": "You are not a player in this game"}).to_string()
        }));
    }

    // Validate that it's the player's turn
    let is_white_turn = matches!(
        game_session.whose_turn,
        shared::models::game_session::Turn::White
    );
    let is_player_white = game_session.player_white_id == player_id;
    let is_player_turn = is_white_turn == is_player_white;

    if !is_player_turn {
        return Ok(json!({
            "statusCode": 400,
            "body": json!({"error": "It's not your turn"}).to_string()
        }));
    }

    // Check if game is still ongoing
    if !matches!(
        game_session.status,
        shared::models::game_session::GameStatus::Ongoing
    ) {
        return Ok(json!({
            "statusCode": 400,
            "body": json!({"error": "Game is not ongoing"}).to_string()
        }));
    }

    // TODO: Implement actual chess move validation and execution
    // For now, just accept the move with basic validation

    Ok(json!({
        "statusCode": 200,
        "body": json!({
            "message": "Move accepted",
            "game_session_id": move_request.game_session_id,
            "move": format!("{} to {}", move_request.from_square, move_request.to_square)
        }).to_string()
    }))
}
