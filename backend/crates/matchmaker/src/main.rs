mod game;
mod matching;
mod models;
mod notifications;

use aws_config::BehaviorVersion;
use aws_lambda_events::event::dynamodb::{Event as DynamoDbEvent, EventRecord};
use aws_sdk_apigatewaymanagement::Client as ApiGatewayClient;
use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_dynamo;
use tracing::{error, info, warn};

use crate::game::attempt_match;
use crate::matching::find_match_for_player;
use crate::models::QueueEntry;
use crate::notifications::notify_player;

#[derive(Clone)]
struct AppState {
    dynamodb: DynamoClient,
    api_gateway: ApiGatewayClient,
    queue_table: String,
    games_table: String,
    connections_table: String,
}

impl AppState {
    async fn new() -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let dynamodb = DynamoClient::new(&config);

        let queue_table = std::env::var("QUEUE_TABLE").expect("QUEUE_TABLE must be set");
        let games_table = std::env::var("GAMES_TABLE").expect("GAMES_TABLE must be set");
        let connections_table =
            std::env::var("CONNECTIONS_TABLE").expect("CONNECTIONS_TABLE must be set");
        let websocket_api_endpoint =
            std::env::var("WEBSOCKET_API_ENDPOINT").expect("WEBSOCKET_API_ENDPOINT must be set");

        let api_config = aws_sdk_apigatewaymanagement::config::Builder::from(&config)
            .endpoint_url(&websocket_api_endpoint)
            .build();
        let api_gateway = ApiGatewayClient::from_conf(api_config);

        info!(
            "Initialized AppState with queue_table={}, games_table={}, connections_table={}",
            queue_table, games_table, connections_table
        );

        Self {
            dynamodb,
            api_gateway,
            queue_table,
            games_table,
            connections_table,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let state = AppState::new().await;
    run(service_fn(|event| handler(event, state.clone()))).await
}

async fn handler(event: LambdaEvent<DynamoDbEvent>, state: AppState) -> Result<(), Error> {
    info!(
        "Received DynamoDB Stream event with {} records",
        event.payload.records.len()
    );

    for record in event.payload.records {
        if let Err(e) = process_record(&state, record).await {
            error!("Failed to process record: {:?}", e);
            // Continue processing other records even if one fails
        }
    }

    Ok(())
}

async fn process_record(state: &AppState, record: EventRecord) -> Result<(), Error> {
    // Only process INSERT events (when a new player joins the queue)
    if record.event_name != "INSERT" {
        info!("Skipping non-INSERT event: {}", record.event_name);
        return Ok(());
    }

    // Extract the new queue entry from the DynamoDB Stream record
    let new_image = match record.change.new_image {
        item if !item.is_empty() => item,
        _ => return Err("No new_image in stream record".into()),
    };

    let new_player: QueueEntry = serde_dynamo::from_item(new_image)?;

    // Skip if player is already matched (in case of duplicate events)
    if new_player.status != "waiting" {
        info!(
            "Player {} is not waiting (status: {}), skipping",
            new_player.user_id, new_player.status
        );
        return Ok(());
    }

    info!(
        "Processing new player in queue: {} (rating: {}, time_control: {}, status: {})",
        new_player.user_id, new_player.rating, new_player.time_control, new_player.status
    );

    // Try to find a match for this player using the new bucket-based algorithm
    // The algorithm will search in expanding ranges (±50, ±100, ..., ±500) with random direction
    loop {
        match find_match_for_player(&state.dynamodb, &state.queue_table, &new_player).await? {
            Some(opponent) => {
                info!(
                    "Found potential opponent: {} (rating: {})",
                    opponent.user_id, opponent.rating
                );

                // Attempt to atomically match both players using DynamoDB transaction
                match attempt_match(
                    &state.dynamodb,
                    &state.queue_table,
                    &state.games_table,
                    &new_player,
                    &opponent,
                )
                .await
                {
                    Ok(game) => {
                        info!(
                            "Successfully matched {} vs {} in game {}",
                            new_player.user_id, opponent.user_id, game.game_id
                        );

                        // Determine colors for each player
                        let (player1_color, player2_color) =
                            if game.white_player_id == new_player.user_id {
                                ("white", "black")
                            } else {
                                ("black", "white")
                            };

                        // Send game_matched notification to both players
                        notify_player(
                            &state.api_gateway,
                            &state.dynamodb,
                            &state.connections_table,
                            &new_player.user_id,
                            &game.game_id,
                            &opponent.user_id,
                            player1_color,
                            &game.time_control,
                        )
                        .await;

                        notify_player(
                            &state.api_gateway,
                            &state.dynamodb,
                            &state.connections_table,
                            &opponent.user_id,
                            &game.game_id,
                            &new_player.user_id,
                            player2_color,
                            &game.time_control,
                        )
                        .await;

                        info!("Match complete, both players notified");
                        return Ok(());
                    }
                    Err(e) => {
                        // Transaction failed - opponent was already matched by another concurrent matchmaking run
                        warn!(
                            "Failed to match with {} (they may have been matched already): {:?}",
                            opponent.user_id, e
                        );

                        // Continue the loop to try finding another opponent
                        info!("Retrying matchmaking for player {}", new_player.user_id);
                        continue;
                    }
                }
            }
            None => {
                // No match found within ±500 points - player stays in queue with status="waiting"
                info!(
                    "No match found for player {} within rating range, they will remain in queue",
                    new_player.user_id
                );
                return Ok(());
            }
        }
    }
}
