use aws_lambda_events::event::dynamodb::{Event, EventRecord};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_dynamo::aws_sdk_dynamodb_1::from_item;
use shared::models::matchmaking::MatchmakingUser;
use tracing::{info, warn};

/// Main handler for DynamoDB Stream events
async fn function_handler(event: LambdaEvent<Event>) -> Result<(), Error> {
    info!(
        "Received DynamoDB Stream event with {} records",
        event.payload.records.len()
    );

    for record in event.payload.records {
        process_record(record).await?;
    }

    Ok(())
}

/// Process a single DynamoDB stream record
async fn process_record(record: EventRecord) -> Result<(), Error> {
    let event_name = record.event_name.as_str();

    match event_name {
        "INSERT" => {
            info!("Processing INSERT event for matchmaking table");

            // Process the new_image directly
            let new_image = record.change.new_image;
            match process_matchmaking_insert(new_image) {
                Ok(_) => info!("Successfully processed matchmaking insert"),
                Err(e) => warn!("Failed to process matchmaking insert: {}", e),
            }
        }
        "MODIFY" => {
            info!("Processing MODIFY event for matchmaking table");
            // Future enhancement: Handle modifications if needed
        }
        "REMOVE" => {
            info!("Processing REMOVE event for matchmaking table");
            // Future enhancement: Handle removals if needed
        }
        _ => {
            warn!("Unknown event type: {}", event_name);
        }
    }

    Ok(())
}

/// Process a DynamoDB INSERT event for matchmaking table
fn process_matchmaking_insert(
    new_image: serde_dynamo::Item,
) -> Result<(), Box<dyn std::error::Error>> {
    // Deserialize the DynamoDB item into our MatchmakingUser model
    let matchmaking_user: MatchmakingUser = from_item(new_image.into())?;

    // Log the item details
    info!(
        "New matchmaking user added - Player ID: {}, Queue Rating: {}, Joined At: {}",
        matchmaking_user.player_id, matchmaking_user.queue_rating, matchmaking_user.joined_at
    );

    // Log the full serialized JSON for debugging
    let json_output = serde_json::to_string_pretty(&matchmaking_user)?;
    info!("Full matchmaking user details:\n{}", json_output);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
