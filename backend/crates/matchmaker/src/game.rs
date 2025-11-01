use aws_sdk_dynamodb::operation::transact_write_items::TransactWriteItemsError;
use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem, Update};
use lambda_runtime::Error;
use serde_dynamo;
use shared::{Game, GameStatus};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::models::QueueEntry;

/// Creates a deterministic game ID based on player IDs and timestamp
/// This ensures idempotency - same players at same time = same game_id
fn create_deterministic_game_id(player1_id: &str, player2_id: &str, timestamp: &str) -> String {
    use sha2::{Digest, Sha256};

    // Sort player IDs to ensure consistency regardless of order
    let mut players = vec![player1_id, player2_id];
    players.sort();

    let input = format!("{}#{}#{}", players[0], players[1], timestamp);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[0..16]) // Use first 16 bytes (32 hex characters)
}

/// Attempts to match two players atomically using DynamoDB transactions
///
/// Transaction includes:
/// 1. Update player1: set status="matched", matched_at=now (with condition: status="waiting")
/// 2. Update player2: set status="matched", matched_at=now (with condition: status="waiting")
/// 3. Create game record
///
/// Returns Ok(Game) if successful, Err if transaction fails (e.g., opponent already matched)
pub async fn attempt_match(
    dynamodb: &aws_sdk_dynamodb::Client,
    queue_table: &str,
    games_table: &str,
    player1: &QueueEntry,
    player2: &QueueEntry,
) -> Result<Game, Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string();

    // Create deterministic game ID
    let game_id = create_deterministic_game_id(&player1.user_id, &player2.user_id, &now);

    info!(
        "Attempting to match {} and {} (game_id: {})",
        player1.user_id, player2.user_id, game_id
    );

    // Randomly assign colors
    let (white_player_id, black_player_id) = if rand::random::<bool>() {
        (player1.user_id.clone(), player2.user_id.clone())
    } else {
        (player2.user_id.clone(), player1.user_id.clone())
    };

    let game = Game {
        game_id: game_id.clone(),
        white_player_id: white_player_id.clone(),
        black_player_id: black_player_id.clone(),
        time_control: player1.time_control.clone(),
        status: GameStatus::Active,
        created_at: now.clone(),
    };

    // Build transaction items
    let transact_items = vec![
        // Update player1 to "matched"
        build_update_player_item(queue_table, player1, &now)?,
        // Update player2 to "matched"
        build_update_player_item(queue_table, player2, &now)?,
        // Create game
        build_create_game_item(games_table, &game)?,
    ];

    // Execute transaction
    match dynamodb
        .transact_write_items()
        .set_transact_items(Some(transact_items))
        .send()
        .await
    {
        Ok(_) => {
            info!(
                "Successfully matched {} and {} in game {}",
                player1.user_id, player2.user_id, game_id
            );
            Ok(game)
        }
        Err(e) => {
            // Check if it's a conditional check failure (player already matched)
            if let Some(service_error) = e.as_service_error() {
                if matches!(
                    service_error,
                    TransactWriteItemsError::TransactionCanceledException(_)
                ) {
                    warn!(
                        "Transaction cancelled - player already matched: {} or {}",
                        player1.user_id, player2.user_id
                    );
                    return Err("Player already matched".into());
                }
            }

            warn!("Transaction failed: {:?}", e);
            Err(e.into())
        }
    }
}

/// Builds a TransactWriteItem to update a player's status to "matched"
fn build_update_player_item(
    queue_table: &str,
    player: &QueueEntry,
    matched_at: &str,
) -> Result<TransactWriteItem, Error> {
    let key = HashMap::from([
        (
            "queue_key".to_string(),
            AttributeValue::S(player.queue_key.clone()),
        ),
        (
            "user_id".to_string(),
            AttributeValue::S(player.user_id.clone()),
        ),
    ]);

    let update = Update::builder()
        .table_name(queue_table)
        .set_key(Some(key))
        .update_expression("SET #status = :matched, matched_at = :matched_at")
        .condition_expression("#status = :waiting AND attribute_not_exists(matched_at)")
        .expression_attribute_names("#status", "status")
        .expression_attribute_values(":matched", AttributeValue::S("matched".to_string()))
        .expression_attribute_values(":waiting", AttributeValue::S("waiting".to_string()))
        .expression_attribute_values(":matched_at", AttributeValue::S(matched_at.to_string()))
        .build()
        .map_err(|e| format!("Failed to build update: {:?}", e))?;

    Ok(TransactWriteItem::builder().update(update).build())
}

/// Builds a TransactWriteItem to create a game record
fn build_create_game_item(games_table: &str, game: &Game) -> Result<TransactWriteItem, Error> {
    let item = serde_dynamo::to_item(game)?;

    let put = Put::builder()
        .table_name(games_table)
        .set_item(Some(item))
        .condition_expression("attribute_not_exists(game_id)") // Prevent duplicate games
        .build()
        .map_err(|e| format!("Failed to build put: {:?}", e))?;

    Ok(TransactWriteItem::builder().put(put).build())
}
