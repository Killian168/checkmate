use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_1::from_item;
use shared::models::game_session::GameSession;
use shared::repositories::websocket_repository::DynamoDbWebSocketRepository;
use shared::services::websocket_service::WebSocketService;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use tracing_subscriber;

#[derive(Serialize, Deserialize)]
struct GameMatchNotification {
    action: String,
    session_id: String,
    opponent_id: String,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    info!("Game session notifier Lambda function starting");
    debug!("Initializing AWS clients for WebSocket notifications");

    // Initialize AWS clients
    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    // Create API Gateway Management API client with correct endpoint
    let websocket_endpoint = format!("https://yphq15v1gk.execute-api.eu-west-1.amazonaws.com/dev");
    let api_gateway_config = aws_sdk_apigatewaymanagement::config::Builder::from(&config)
        .endpoint_url(&websocket_endpoint)
        .build();
    let api_gateway_client = aws_sdk_apigatewaymanagement::Client::from_conf(api_gateway_config);

    // Initialize WebSocket service
    let websocket_repository = Arc::new(DynamoDbWebSocketRepository::new(
        dynamodb_client,
        api_gateway_client,
    ));
    let websocket_service = Arc::new(WebSocketService::new(websocket_repository));
    debug!("WebSocket service initialized successfully");

    run(service_fn(move |event: LambdaEvent<Event>| {
        let websocket_service = websocket_service.clone();
        async move {
            let (event, _context) = event.into_parts();

            info!("Processing {} records", event.records.len());

            for (i, record) in event.records.iter().enumerate() {
                debug!(
                    "Processing record {} with event_name: {}",
                    i, record.event_name
                );
                if let Err(e) = process_record(record.clone(), &websocket_service).await {
                    error!("Failed to process record {}: {}", i, e);
                }
            }

            Ok::<(), Error>(())
        }
    }))
    .await
}

async fn process_record(
    record: aws_lambda_events::event::dynamodb::EventRecord,
    websocket_service: &WebSocketService,
) -> Result<(), Box<dyn std::error::Error>> {
    match record.event_name.as_str() {
        "INSERT" => {
            let new_image = record.change.new_image;
            debug!("Processing INSERT event, new_image: {:?}", new_image);
            let game_session: GameSession = from_item(new_image.into())?;
            debug!("Successfully parsed game session: {:?}", game_session);

            info!(
                "New game session created: {} between {} and {}",
                game_session.session_id, game_session.player1_id, game_session.player2_id
            );

            // Notify both players about the match
            debug!("Starting player notification process");
            notify_players(&game_session, websocket_service).await?;
        }
        _ => {
            info!("Unhandled event type: {}", record.event_name);
        }
    }

    Ok(())
}

async fn notify_players(
    game_session: &GameSession,
    websocket_service: &WebSocketService,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create notification messages for both players
    let notification_player1 = GameMatchNotification {
        action: "game_matched".to_string(),
        session_id: game_session.session_id.clone(),
        opponent_id: game_session.player2_id.clone(),
        message: format!(
            "You have been matched against player {}",
            game_session.player2_id
        ),
    };

    let notification_player2 = GameMatchNotification {
        action: "game_matched".to_string(),
        session_id: game_session.session_id.clone(),
        opponent_id: game_session.player1_id.clone(),
        message: format!(
            "You have been matched against player {}",
            game_session.player1_id
        ),
    };

    // Convert notifications to JSON
    let message_player1 = serde_json::to_string(&notification_player1)?;
    let message_player2 = serde_json::to_string(&notification_player2)?;

    // Send notifications to both players
    info!(
        "Sending match notification to player {}",
        game_session.player1_id
    );
    debug!("Player 1 notification message: {}", message_player1);
    if let Err(e) = websocket_service
        .send_notification(&game_session.player1_id, &message_player1)
        .await
    {
        error!("Failed to notify player {}: {}", game_session.player1_id, e);
        warn!("Player 1 may not be connected to WebSocket");
    } else {
        debug!("Successfully sent notification to player 1");
    }

    info!(
        "Sending match notification to player {}",
        game_session.player2_id
    );
    debug!("Player 2 notification message: {}", message_player2);
    if let Err(e) = websocket_service
        .send_notification(&game_session.player2_id, &message_player2)
        .await
    {
        error!("Failed to notify player {}: {}", game_session.player2_id, e);
        warn!("Player 2 may not be connected to WebSocket");
    } else {
        debug!("Successfully sent notification to player 2");
    }

    info!(
        "Successfully notified both players about game session {}",
        game_session.session_id
    );

    Ok(())
}
