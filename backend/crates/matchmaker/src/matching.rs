use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_runtime::Error;
use rand::seq::SliceRandom;
use rand::Rng;
use serde_dynamo;
use tracing::{info, warn};

use crate::models::QueueEntry;

const RANGE_STEP: i32 = 50;
const MAX_RANGE: i32 = 500;

/// Normalizes a rating to the nearest bucket (floors to nearest 50)
pub fn normalize_rating(rating: i32) -> i32 {
    (rating / RANGE_STEP) * RANGE_STEP
}

/// Finds a match for a player using the new bucket-based algorithm with random search direction
///
/// Algorithm:
/// 1. Compute player's rating bucket (rating // 50) * 50
/// 2. Randomly decide whether to start searching upward or downward
/// 3. Expand search range gradually: 50 → 100 → 150 → ... → 500
/// 4. For each expansion level, shuffle offsets and try both directions
/// 5. Query DynamoDB for candidates with status = "waiting"
/// 6. Randomly pick an opponent from results
/// 7. Return the first valid candidate found
pub async fn find_match_for_player(
    dynamodb: &DynamoClient,
    queue_table: &str,
    new_player: &QueueEntry,
) -> Result<Option<QueueEntry>, Error> {
    info!(
        "Finding match for player {} (rating: {}, time_control: {})",
        new_player.user_id, new_player.rating, new_player.time_control
    );

    let player_bucket = normalize_rating(new_player.rating);
    info!("Player bucket: {}", player_bucket);

    // First, check the player's own bucket (offset 0)
    let queue_key = format!("{}#{}", new_player.time_control, player_bucket);
    info!("First checking own bucket: {}", queue_key);

    match query_bucket(dynamodb, queue_table, &queue_key, &new_player.user_id).await? {
        Some(candidates) if !candidates.is_empty() => {
            info!(
                "Found {} candidates in own bucket {}",
                candidates.len(),
                queue_key
            );
            let mut rng = rand::thread_rng();
            if let Some(opponent) = candidates.choose(&mut rng) {
                info!(
                    "Selected opponent: {} (rating: {}) from own bucket",
                    opponent.user_id, opponent.rating
                );
                return Ok(Some(opponent.clone()));
            }
        }
        _ => {
            info!("No candidates in own bucket {}", queue_key);
        }
    }

    // Randomly decide whether to start searching upward (+1) or downward (-1)
    let mut rng = rand::thread_rng();
    let start_direction = if rng.gen_bool(0.5) { 1 } else { -1 };
    info!(
        "Search direction: {}",
        if start_direction == 1 {
            "upward"
        } else {
            "downward"
        }
    );

    // Expand search range gradually: 50 → 100 → 150 → ... → 500
    for range in (RANGE_STEP..=MAX_RANGE).step_by(RANGE_STEP as usize) {
        info!("Searching at range ±{}", range);

        // Create offsets for this range
        let mut offsets = vec![range * start_direction, range * -start_direction];
        offsets.shuffle(&mut rng);

        for offset in offsets {
            let candidate_bucket = normalize_rating(new_player.rating + offset);
            let queue_key = format!("{}#{}", new_player.time_control, candidate_bucket);

            info!(
                "Querying bucket: {} (offset: {}, candidate_bucket: {})",
                queue_key, offset, candidate_bucket
            );

            // Query for waiting players in this bucket
            match query_bucket(dynamodb, queue_table, &queue_key, &new_player.user_id).await? {
                Some(candidates) if !candidates.is_empty() => {
                    info!(
                        "Found {} candidates in bucket {}",
                        candidates.len(),
                        queue_key
                    );

                    // Randomly pick one opponent
                    if let Some(opponent) = candidates.choose(&mut rng) {
                        info!(
                            "Selected opponent: {} (rating: {})",
                            opponent.user_id, opponent.rating
                        );
                        return Ok(Some(opponent.clone()));
                    }
                }
                _ => {
                    info!("No candidates in bucket {}", queue_key);
                }
            }
        }
    }

    info!(
        "No match found for player {} within ±{} points",
        new_player.user_id, MAX_RANGE
    );
    Ok(None)
}

/// Queries a specific rating bucket for waiting players
async fn query_bucket(
    dynamodb: &DynamoClient,
    queue_table: &str,
    queue_key: &str,
    exclude_user_id: &str,
) -> Result<Option<Vec<QueueEntry>>, Error> {
    let query_result = dynamodb
        .query()
        .table_name(queue_table)
        .key_condition_expression("queue_key = :qk")
        .filter_expression("#status = :waiting")
        .expression_attribute_names("#status", "status")
        .expression_attribute_values(":qk", AttributeValue::S(queue_key.to_string()))
        .expression_attribute_values(":waiting", AttributeValue::S("waiting".to_string()))
        .send()
        .await?;

    let items = query_result.items.unwrap_or_default();

    if items.is_empty() {
        return Ok(None);
    }

    // Parse and filter out the current player
    let candidates: Vec<QueueEntry> = items
        .into_iter()
        .filter_map(|item| match serde_dynamo::from_item(item) {
            Ok(entry) => Some(entry),
            Err(e) => {
                warn!("Failed to parse queue entry: {:?}", e);
                None
            }
        })
        .filter(|entry: &QueueEntry| entry.user_id != exclude_user_id)
        .collect();

    if candidates.is_empty() {
        Ok(None)
    } else {
        Ok(Some(candidates))
    }
}
