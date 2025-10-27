use rand::seq::SliceRandom;
use std::sync::Arc;

use crate::{
    models::queue::QueueUser, models::user::User, repositories::queue_repository::QueueRepository,
    services::errors::queue_service_errors::QueueServiceError,
};

#[derive(Clone)]
pub struct QueueService {
    repository: Arc<dyn QueueRepository + Send + Sync>,
}

impl QueueService {
    pub fn new(repository: Arc<dyn QueueRepository + Send + Sync>) -> Self {
        QueueService { repository }
    }

    pub async fn join_queue(
        &self,
        player: &User,
        queue_type: &str,
    ) -> Result<QueueUser, QueueServiceError> {
        let player_id = &player.id;

        if queue_type.is_empty() {
            return Err(QueueServiceError::ValidationError(
                "Queue type cannot be empty".to_string(),
            ));
        }

        let user = QueueUser::new(player_id, player.rating, queue_type);

        self.repository
            .join_queue(player_id, queue_type, player.rating)
            .await
            .map_err(|e| QueueServiceError::RepositoryError(e.to_string()))?;

        Ok(user)
    }

    pub async fn leave_queue(
        &self,
        player: &User,
        queue_type: &str,
    ) -> Result<(), QueueServiceError> {
        let player_id = &player.id;

        if queue_type.is_empty() {
            return Err(QueueServiceError::ValidationError(
                "Queue type cannot be empty".to_string(),
            ));
        }

        self.repository
            .leave_queue(player_id, queue_type, player.rating)
            .await
            .map_err(|e| QueueServiceError::RepositoryError(e.to_string()))
    }

    pub async fn find_opponent(
        &self,
        user: &QueueUser,
    ) -> Result<Option<QueueUser>, QueueServiceError> {
        let user_rating = user.rating();
        let queue_type = user.queue_type();

        // Progressive search ranges: ±100, ±200, ±300, ±400, ±500
        let search_ranges = vec![0, 100, 200, 300, 400, 500];

        for range in search_ranges {
            if let Some(opponent) = self
                .search_in_range(user, user_rating, &queue_type, range)
                .await?
            {
                return Ok(Some(opponent));
            }
        }

        Ok(None)
    }

    async fn search_in_range(
        &self,
        user: &QueueUser,
        user_rating: i32,
        queue_type: &str,
        range: i32,
    ) -> Result<Option<QueueUser>, QueueServiceError> {
        // Generate rating buckets within the current range
        let lower_range = user_rating - range;
        let upper_range = user_rating + range;
        let mut rating_buckets = vec![lower_range, upper_range];

        let mut rng = rand::thread_rng();
        rating_buckets.shuffle(&mut rng);

        // Try each bucket in random order
        for bucket in rating_buckets {
            // Get potential opponents from this bucket
            let mut bucket_opponents = self
                .repository
                .find_potential_opponents(&user.player_id, queue_type, bucket)
                .await
                .map_err(|e| QueueServiceError::RepositoryError(e.to_string()))?;

            // Sort opponents by joined_at to ensure fairness
            bucket_opponents.sort_by_key(|opponent| opponent.joined_at);

            // Try each opponent in order
            for opponent in bucket_opponents {
                // Atomically reserve the opponent to prevent race conditions
                if self
                    .repository
                    .reserve_opponent(&opponent)
                    .await
                    .map_err(|e| QueueServiceError::RepositoryError(e.to_string()))?
                {
                    return Ok(Some(opponent));
                }
            }
        }

        Ok(None)
    }
}
