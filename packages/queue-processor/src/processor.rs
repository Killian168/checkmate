use aws_lambda_events::event::dynamodb::Event;
use lambda_runtime::Error;
use serde_dynamo::aws_sdk_dynamodb_1::from_item;
use shared::models::queue::QueueUser;
use shared::services::game_session_service::GameSessionService;
use shared::services::queue_service::QueueService;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct QueueProcessor {
    queue_service: QueueService,
    game_session_service: GameSessionService,
}

impl QueueProcessor {
    pub fn new(queue_service: QueueService, game_session_service: GameSessionService) -> Self {
        Self {
            queue_service,
            game_session_service,
        }
    }

    pub async fn process_event(&self, event: Event) -> Result<(), Error> {
        // Process the first record (or only record in single-record batches)
        if let Some(record) = event.records.into_iter().next() {
            let event_name = record.event_name.as_str();

            match event_name {
                "INSERT" => {
                    let new_image = record.change.new_image;
                    let queue_user: QueueUser = from_item(new_image.into())?;

                    info!(
                        "Processing new user in Queue queue: {} (rating bucket: {})",
                        queue_user.player_id, queue_user.queue_rating
                    );

                    match self.queue_service.find_opponent(&queue_user).await {
                        Ok(Some(opponent)) => {
                            info!("Opponent found for user: {}", queue_user.player_id);
                            info!("Opponent ID: {}", opponent.player_id);
                            self.game_session_service
                                .create_game_session(&queue_user, &opponent)
                                .await?;
                        }
                        Ok(None) => {
                            info!(
                                "No suitable opponent found for user: {}. User remains in queue.",
                                queue_user.player_id
                            );
                        }
                        Err(e) => {
                            error!("Queue error for user {}: {}", queue_user.player_id, e);
                        }
                    }
                }
                _ => {
                    warn!("Unhandled event type: {}", event_name);
                }
            }
        }

        Ok(())
    }
}
