use std::sync::Arc;

use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::Error;
use serde_dynamo::aws_sdk_dynamodb_1::from_item;
use shared::models::matchmaking::MatchmakingUser;
use tracing::{error, info, warn};

use crate::{
    repositories::matchmaking_repository::DynamoDbMatchmakingRepository,
    services::MatchmakingService,
};

#[derive(Clone)]
pub struct MatchmakingProcessor {
    service: MatchmakingService,
}

impl MatchmakingProcessor {
    pub fn new(client: aws_sdk_dynamodb::Client) -> Self {
        let repository = Arc::new(DynamoDbMatchmakingRepository::new(client));
        let service = MatchmakingService::new(repository);
        Self { service }
    }

    pub async fn process_event(&self, event: Event) -> Result<(), Error> {
        // Validate batch size assumption - we expect exactly one record per invocation
        if event.records.len() != 1 {
            warn!(
                "Unexpected batch size: expected 1 record, got {}. Processing first record only.",
                event.records.len()
            );
        }

        // Process the first record (or only record in single-record batches)
        if let Some(record) = event.records.into_iter().next() {
            let event_name = record.event_name.as_str();

            match event_name {
                "INSERT" => {
                    let new_image = record.change.new_image;
                    let matchmaking_user: MatchmakingUser = from_item(new_image.into())?;

                    info!(
                        "Processing new user in matchmaking queue: {} (rating bucket: {})",
                        matchmaking_user.player_id, matchmaking_user.queue_rating
                    );

                    if let Err(e) = self.process_new_user(&matchmaking_user).await {
                        error!(
                            "Failed to process matchmaking user {}: {}",
                            matchmaking_user.player_id, e
                        );
                    }
                }
                "REMOVE" => {
                    // Handle user leaving the queue (optional cleanup or analytics)
                    let old_image = record.change.old_image;
                    if let Ok(matchmaking_user) = from_item::<MatchmakingUser>(old_image.into()) {
                        info!(
                            "User {} left the matchmaking queue",
                            matchmaking_user.player_id
                        );
                    }
                }
                _ => {
                    warn!("Unhandled event type: {}", event_name);
                }
            }
        }

        Ok(())
    }

    async fn process_new_user(&self, user: &MatchmakingUser) -> Result<(), Error> {
        match self.service.find_and_match_opponent(user).await {
            Ok(Some(game_session)) => {
                info!(
                    "Successfully matched {} with {}",
                    game_session.player1_id, game_session.player2_id
                );

                // Log the game session (in production, you might store this in a separate table)
                let session_json = serde_json::to_string_pretty(&game_session)?;
                info!("Game session created:\n{}", session_json);
            }
            Ok(None) => {
                info!(
                    "No suitable opponent found for user: {}. User remains in queue.",
                    user.player_id
                );
            }
            Err(e) => {
                error!("Matchmaking error for user {}: {}", user.player_id, e);
            }
        }

        Ok(())
    }
}
