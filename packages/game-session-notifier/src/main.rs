use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_dynamo::aws_sdk_dynamodb_1::from_item;
use shared::models::game_session::GameSession;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    info!("Game session notifier Lambda function starting");

    run(service_fn(|event: LambdaEvent<Event>| async {
        let (event, _context) = event.into_parts();

        info!("Processing {} records", event.records.len());

        for record in event.records {
            if let Err(e) = process_record(record).await {
                error!("Failed to process record: {}", e);
            }
        }

        Ok::<(), Error>(())
    }))
    .await
}

async fn process_record(
    record: aws_lambda_events::event::dynamodb::EventRecord,
) -> Result<(), Box<dyn std::error::Error>> {
    match record.event_name.as_str() {
        "INSERT" => {
            let new_image = record.change.new_image;
            let game_session: GameSession = from_item(new_image.into())?;
            info!(
                "New game session created: {} between {} and {}",
                game_session.session_id, game_session.player1_id, game_session.player2_id
            );
        }
        _ => {
            info!("Unhandled event type: {}", record.event_name);
        }
    }

    Ok(())
}
