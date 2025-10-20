use lambda_runtime::{run, service_fn, Error};
use std::sync::Arc;
use tracing_subscriber;

mod processor;
use processor::QueueProcessor;
use shared::{
    repositories::{
        game_repository::DynamoDbGameSessionRepository, queue_repository::DynamoDbQueueRepository,
    },
    services::{game_session_service::GameSessionService, queue_service::QueueService},
};

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

    // Create services
    let queue_repository = Arc::new(DynamoDbQueueRepository::new(client.clone()));
    let queue_service = QueueService::new(queue_repository);

    let game_session_repository = Arc::new(DynamoDbGameSessionRepository::new(client.clone()));
    let game_session_service = GameSessionService::new(game_session_repository);

    // Create processor with services
    let processor = QueueProcessor::new(queue_service, game_session_service);

    // Run the Lambda function
    run(service_fn(
        move |event: lambda_runtime::LambdaEvent<aws_lambda_events::event::dynamodb::Event>| {
            let processor = processor.clone();
            async move { processor.process_event(event.payload).await }
        },
    ))
    .await
}
