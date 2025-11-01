use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::Error;
use serde_dynamo;
use std::collections::HashMap;
use tracing::info;

use crate::models::{JoinQueueMessage, QueueEntry};
use crate::AppState;

pub async fn join_queue(
    state: &AppState,
    user_id: &str,
    msg: &JoinQueueMessage,
) -> Result<(), Error> {
    info!(
        "Joining queue for user {} with time_control {}, min_rating {:?}, max_rating {:?}",
        user_id, msg.time_control, msg.min_rating, msg.max_rating
    );
    // Get user's rating
    let users_table = std::env::var("USERS_TABLE").expect("USERS_TABLE must be set");
    info!(
        "Fetching rating for user {} from table {}",
        user_id, users_table
    );
    let user_item = state
        .dynamodb
        .get_item()
        .table_name(&users_table)
        .key("user_id", AttributeValue::S(user_id.to_string()))
        .send()
        .await?;

    let rating = if let Some(item) = user_item.item {
        if let Some(AttributeValue::N(r)) = item.get("rating") {
            r.parse::<i32>().unwrap_or(1200)
        } else {
            1200
        }
    } else {
        1200
    };

    info!("User {} has rating {}", user_id, rating);
    let rating_bucket = ((rating / 50) * 50).to_string();
    let pk = format!("{}#{}", msg.time_control, rating_bucket);
    info!(
        "Calculated rating bucket {} for time_control {}, queue_key {}",
        rating_bucket, msg.time_control, pk
    );

    // Check if already in queue
    let key = HashMap::from([
        ("queue_key".to_string(), AttributeValue::S(pk.clone())),
        (
            "user_id".to_string(),
            AttributeValue::S(user_id.to_string()),
        ),
    ]);
    info!(
        "Checking if user {} is already in queue for key {}",
        user_id, pk
    );
    let existing = state
        .dynamodb
        .get_item()
        .table_name(&state.queue_table)
        .set_key(Some(key))
        .send()
        .await?;

    if existing.item.is_some() {
        info!("User {} already in queue for key {}", user_id, pk);
        return Err("Already in queue".into());
    }
    info!("User {} not in queue, proceeding to join", user_id);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string();

    let entry = QueueEntry {
        queue_key: pk.clone(),
        user_id: user_id.to_string(),
        time_control: msg.time_control.clone(),
        rating_bucket,
        rating,
        joined_at: now.clone(),
        min_rating: msg.min_rating,
        max_rating: msg.max_rating,
    };

    info!(
        "Creating queue entry for user {}: queue_key={}, time_control={}, rating_bucket={}, rating={}, joined_at={}",
        user_id, entry.queue_key, entry.time_control, entry.rating_bucket, entry.rating, entry.joined_at
    );
    let item = serde_dynamo::to_item(&entry)?;
    info!("Storing queue entry in table {}", state.queue_table);
    state
        .dynamodb
        .put_item()
        .table_name(&state.queue_table)
        .set_item(Some(item))
        .send()
        .await?;
    info!(
        "Successfully joined queue for user {} with queue_key {}",
        user_id, pk
    );
    Ok(())
}

pub async fn leave_queue(state: &AppState, user_id: &str, time_control: &str) -> Result<(), Error> {
    info!(
        "Leaving queue for user {} with time_control {}",
        user_id, time_control
    );
    // Get user's rating to compute the bucket
    let users_table = std::env::var("USERS_TABLE").expect("USERS_TABLE must be set");
    info!(
        "Fetching rating for user {} from table {}",
        user_id, users_table
    );
    let user_item = state
        .dynamodb
        .get_item()
        .table_name(&users_table)
        .key("user_id", AttributeValue::S(user_id.to_string()))
        .send()
        .await?;

    let rating = if let Some(item) = user_item.item {
        if let Some(AttributeValue::N(r)) = item.get("rating") {
            r.parse::<i32>().unwrap_or(1200)
        } else {
            1200
        }
    } else {
        1200
    };

    info!("User {} has rating {} for leaving queue", user_id, rating);
    let rating_bucket = ((rating / 50) * 50).to_string();
    let pk = format!("{}#{}", time_control, rating_bucket);
    info!(
        "Calculated queue_key {} for user {} leaving queue with time_control {}",
        pk, user_id, time_control
    );

    let key = HashMap::from([
        ("queue_key".to_string(), AttributeValue::S(pk.clone())),
        (
            "user_id".to_string(),
            AttributeValue::S(user_id.to_string()),
        ),
    ]);

    info!("Removing user {} from queue with key {}", user_id, pk);
    state
        .dynamodb
        .delete_item()
        .table_name(&state.queue_table)
        .set_key(Some(key))
        .send()
        .await?;
    info!(
        "Successfully removed user {} from queue with key {}",
        user_id, pk
    );
    Ok(())
}
