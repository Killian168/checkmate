use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_dynamo::aws_sdk_dynamodb_1::from_item;
use shared::models::matchmaking::MatchmakingUser;
use tracing::{info, warn};

async fn function_handler(event: LambdaEvent<Event>) -> Result<(), Error> {
    if event.payload.records.len() > 1 {
        let message = "Received more than one record in a single event";
        warn!("{}", message);
        return Ok(());
    }

    let record = event.payload.records[0].clone();
    let event_name = record.event_name.as_str();

    match event_name {
        "INSERT" => {
            let new_image = record.change.new_image;
            let matchmaking_user: MatchmakingUser = from_item(new_image.into())?;
            let json_output = serde_json::to_string_pretty(&matchmaking_user)?;
            info!("Full matchmaking user details:\n{}", json_output);
        }
        _ => {
            warn!("Unhandled event type: {}", event_name);
        }
    }

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
