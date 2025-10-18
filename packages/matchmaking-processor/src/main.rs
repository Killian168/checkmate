use lambda_runtime::{run, service_fn, Error};
use tracing_subscriber;

mod models;
mod processor;
mod repositories;
mod services;

use processor::MatchmakingProcessor;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    // Set up AWS configuration and processor
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let processor = MatchmakingProcessor::new(client);

    // Run the Lambda function
    run(service_fn(
        move |event: lambda_runtime::LambdaEvent<aws_lambda_events::event::dynamodb::Event>| {
            let processor = processor.clone();
            async move { processor.process_event(event.payload).await }
        },
    ))
    .await
}
