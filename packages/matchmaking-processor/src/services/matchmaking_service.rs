use std::sync::Arc;

use shared::models::matchmaking::MatchmakingUser;

use crate::{
    models::{extract_queue_type_from_queue_rating, extract_rating_from_queue_rating, GameSession},
    repositories::MatchmakingRepository,
    services::errors::MatchmakingServiceError,
};

#[derive(Clone)]
pub struct MatchmakingService {
    repository: Arc<dyn MatchmakingRepository + Send + Sync>,
}

impl MatchmakingService {
    pub fn new(repository: Arc<dyn MatchmakingRepository + Send + Sync>) -> Self {
        MatchmakingService { repository }
    }

    pub async fn find_and_match_opponent(
        &self,
        user: &MatchmakingUser,
    ) -> Result<Option<GameSession>, MatchmakingServiceError> {
        let user_rating = extract_rating_from_queue_rating(&user.queue_rating);
        let queue_type = extract_queue_type_from_queue_rating(&user.queue_rating);

        // Progressive search ranges: ±100, ±200, ±300, ±400, ±500
        let search_ranges = vec![100, 200, 300, 400, 500];

        for range in search_ranges {
            if let Some(game_session) = self
                .search_in_range(user, user_rating, &queue_type, range)
                .await?
            {
                return Ok(Some(game_session));
            }
        }

        Ok(None)
    }

    async fn search_in_range(
        &self,
        user: &MatchmakingUser,
        user_rating: i32,
        queue_type: &str,
        range: i32,
    ) -> Result<Option<GameSession>, MatchmakingServiceError> {
        // Generate rating buckets within the current range
        let mut rating_buckets = Vec::new();

        // Add buckets from -range to +range in steps of 100
        for offset in (-range..=range).step_by(100) {
            let bucket = user_rating + offset;
            if bucket >= 0 {
                // Skip negative ratings
                rating_buckets.push(bucket);
            }
        }

        // Remove duplicates and shuffle for fair distribution
        rating_buckets.sort();
        rating_buckets.dedup();

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        rating_buckets.shuffle(&mut rng);

        // Try each bucket in random order
        for bucket in rating_buckets {
            let queue_rating = format!("{}#{}", queue_type, bucket);

            // Get potential opponents from this bucket
            let bucket_opponents = self
                .repository
                .find_potential_opponents(&queue_rating, &user.player_id)
                .await?;

            if let Some(opponent) = self.select_best_opponent(&bucket_opponents, user).await? {
                // Atomically reserve the opponent to prevent race conditions
                if self
                    .repository
                    .check_and_reserve_opponent(&opponent)
                    .await?
                {
                    let game_session = GameSession::new(
                        &user.player_id,
                        &opponent.player_id,
                        user_rating,
                        extract_rating_from_queue_rating(&opponent.queue_rating),
                        queue_type,
                    );

                    self.repository
                        .create_game_session(user, &opponent, &game_session)
                        .await?;

                    return Ok(Some(game_session));
                }
                // If reservation failed, opponent was already taken by another process
                // Continue searching for other opponents
            }
        }

        Ok(None)
    }

    async fn select_best_opponent(
        &self,
        opponents: &[MatchmakingUser],
        _user: &MatchmakingUser,
    ) -> Result<Option<MatchmakingUser>, MatchmakingServiceError> {
        if opponents.is_empty() {
            return Ok(None);
        }

        // Select the opponent who has been waiting the longest
        // This ensures fairness and prevents players from waiting indefinitely
        let best_opponent = opponents
            .iter()
            .min_by_key(|opponent| opponent.joined_at)
            .cloned();

        Ok(best_opponent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::matchmaking_repository::tests::MockMatchmakingRepository;
    use async_trait::async_trait;

    use chrono::Utc;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_and_match_opponent_success() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let opponent = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user2".to_string(),
            joined_at: Utc::now(),
        };

        let mock_repository =
            Arc::new(MockMatchmakingRepository::new().with_opponents(vec![opponent.clone()]));
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_some());
        let game_session = result.unwrap();
        assert_eq!(game_session.player1_id, "user1");
        assert_eq!(game_session.player2_id, "user2");
        assert_eq!(game_session.queue_type, "rapid");
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_no_opponents() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let mock_repository = Arc::new(MockMatchmakingRepository::new());
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_excludes_current_user() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        // Only include the current user in opponents (should be excluded)
        let mock_repository =
            Arc::new(MockMatchmakingRepository::new().with_opponents(vec![user.clone()]));
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_select_best_opponent_returns_longest_waiting() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let now = Utc::now();
        let opponent1 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(5), // Joined 5 minutes ago
        };

        let opponent2 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user3".to_string(),
            joined_at: now - chrono::Duration::minutes(10), // Joined 10 minutes ago (longest waiting)
        };

        let service = MatchmakingService::new(Arc::new(MockMatchmakingRepository::new()));

        let opponents = vec![opponent1.clone(), opponent2.clone()];
        let result = service
            .select_best_opponent(&opponents, &user)
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().player_id, "user3"); // Should return longest waiting opponent
    }

    #[tokio::test]
    async fn test_select_best_opponent_empty_list() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let service = MatchmakingService::new(Arc::new(MockMatchmakingRepository::new()));

        let result = service.select_best_opponent(&[], &user).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_prioritizes_longest_waiting() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let now = Utc::now();
        let opponent1 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(2),
        };

        let opponent2 = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user3".to_string(),
            joined_at: now - chrono::Duration::minutes(5), // Longest waiting
        };

        let mock_repository = Arc::new(
            MockMatchmakingRepository::new()
                .with_opponents(vec![opponent1.clone(), opponent2.clone()]),
        );
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_some());
        let game_session = result.unwrap();
        assert_eq!(game_session.player1_id, "user1");
        assert_eq!(game_session.player2_id, "user3"); // Should match with longest waiting opponent
        assert_eq!(game_session.queue_type, "rapid");
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_expands_search_range() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let now = Utc::now();
        // No opponents in ±100 range, but opponent in ±200 range
        let opponent = MatchmakingUser {
            queue_rating: "rapid#1600".to_string(), // 200 points higher
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(3),
        };

        // Mock repository that returns different opponents based on bucket
        let mock_repository = Arc::new(MockMatchmakingRepository::new().with_opponents(vec![
            // ±100 range: empty
            // ±200 range: has opponent
            opponent.clone(),
        ]));
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_some());
        let game_session = result.unwrap();
        assert_eq!(game_session.player1_id, "user1");
        assert_eq!(game_session.player2_id, "user2");
        assert_eq!(game_session.player1_rating, 1400);
        assert_eq!(game_session.player2_rating, 1600);
        assert_eq!(game_session.queue_type, "rapid");
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_no_opponents_in_any_range() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        // Empty repository - no opponents in any range
        let mock_repository = Arc::new(MockMatchmakingRepository::new());
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_high_rated_player_finds_match() {
        let user = MatchmakingUser {
            queue_rating: "rapid#2800".to_string(), // Very high rated player
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let now = Utc::now();
        // Opponent found in expanded range (±500)
        let opponent = MatchmakingUser {
            queue_rating: "rapid#2300".to_string(), // 500 points lower
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(10),
        };

        let mock_repository =
            Arc::new(MockMatchmakingRepository::new().with_opponents(vec![opponent.clone()]));
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_some());
        let game_session = result.unwrap();
        assert_eq!(game_session.player1_id, "user1");
        assert_eq!(game_session.player2_id, "user2");
        assert_eq!(game_session.player1_rating, 2800);
        assert_eq!(game_session.player2_rating, 2300);
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_low_rated_player_finds_match() {
        let user = MatchmakingUser {
            queue_rating: "rapid#100".to_string(), // Very low rated player
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let now = Utc::now();
        // Opponent found in expanded range (±500)
        let opponent = MatchmakingUser {
            queue_rating: "rapid#500".to_string(), // 400 points higher
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(8),
        };

        let mock_repository =
            Arc::new(MockMatchmakingRepository::new().with_opponents(vec![opponent.clone()]));
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        assert!(result.is_some());
        let game_session = result.unwrap();
        assert_eq!(game_session.player1_id, "user1");
        assert_eq!(game_session.player2_id, "user2");
        assert_eq!(game_session.player1_rating, 100);
        assert_eq!(game_session.player2_rating, 500);
    }

    #[tokio::test]
    async fn test_search_in_range_finds_opponent_in_same_bucket() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let now = Utc::now();
        let opponent = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user2".to_string(),
            joined_at: now - chrono::Duration::minutes(5),
        };

        let mock_repository =
            Arc::new(MockMatchmakingRepository::new().with_opponents(vec![opponent.clone()]));
        let service = MatchmakingService::new(mock_repository);

        let result = service
            .search_in_range(&user, 1400, "rapid", 100)
            .await
            .unwrap();

        assert!(result.is_some());
        let game_session = result.unwrap();
        assert_eq!(game_session.player1_id, "user1");
        assert_eq!(game_session.player2_id, "user2");
        assert_eq!(game_session.player1_rating, 1400);
        assert_eq!(game_session.player2_rating, 1400);
    }

    #[tokio::test]
    async fn test_search_in_range_no_opponents_in_range() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        // Opponent outside the search range
        let opponent = MatchmakingUser {
            queue_rating: "rapid#2000".to_string(), // Outside ±100 range
            player_id: "user2".to_string(),
            joined_at: Utc::now() - chrono::Duration::minutes(5),
        };

        let mock_repository =
            Arc::new(MockMatchmakingRepository::new().with_opponents(vec![opponent.clone()]));
        let service = MatchmakingService::new(mock_repository);

        let result = service
            .search_in_range(&user, 1400, "rapid", 100)
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_handles_reservation_failure() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let opponent = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "opponent".to_string(),
            joined_at: Utc::now() - chrono::Duration::minutes(5),
        };

        // Mock repository that always fails to reserve opponents
        let mock_repository = Arc::new(FailingReservationMockRepository::new(
            vec![opponent.clone()],
            false, // Always fail reservations
        ));
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        // Should not match since reservation always fails
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_and_match_opponent_succeeds_when_reservation_works() {
        let user = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "user1".to_string(),
            joined_at: Utc::now(),
        };

        let opponent = MatchmakingUser {
            queue_rating: "rapid#1400".to_string(),
            player_id: "opponent".to_string(),
            joined_at: Utc::now() - chrono::Duration::minutes(5),
        };

        // Mock repository that always succeeds in reserving opponents
        let mock_repository = Arc::new(FailingReservationMockRepository::new(
            vec![opponent.clone()],
            true, // Always succeed reservations
        ));
        let service = MatchmakingService::new(mock_repository);

        let result = service.find_and_match_opponent(&user).await.unwrap();

        // Should match since reservation succeeds
        assert!(result.is_some());
        let game_session = result.unwrap();
        assert_eq!(game_session.player1_id, "user1");
        assert_eq!(game_session.player2_id, "opponent");
    }

    // Simple mock repository for testing reservation behavior
    #[derive(Clone)]
    struct FailingReservationMockRepository {
        opponents: Vec<MatchmakingUser>,
        reservation_succeeds: bool,
    }

    impl FailingReservationMockRepository {
        pub fn new(opponents: Vec<MatchmakingUser>, reservation_succeeds: bool) -> Self {
            Self {
                opponents,
                reservation_succeeds,
            }
        }
    }

    #[async_trait]
    impl MatchmakingRepository for FailingReservationMockRepository {
        async fn find_potential_opponents(
            &self,
            queue_rating: &str,
            excluded_player_id: &str,
        ) -> Result<
            Vec<MatchmakingUser>,
            crate::repositories::matchmaking_repository::MatchmakingRepositoryError,
        > {
            let mut opponents: Vec<MatchmakingUser> = self
                .opponents
                .iter()
                .filter(|opponent| opponent.player_id != excluded_player_id)
                .filter(|opponent| opponent.queue_rating == queue_rating)
                .cloned()
                .collect();

            // Sort opponents by join time (oldest first)
            opponents.sort_by_key(|opponent| opponent.joined_at);

            Ok(opponents)
        }

        async fn create_game_session(
            &self,
            _player1: &MatchmakingUser,
            _player2: &MatchmakingUser,
            _game_session: &GameSession,
        ) -> Result<(), crate::repositories::matchmaking_repository::MatchmakingRepositoryError>
        {
            Ok(())
        }

        async fn check_and_reserve_opponent(
            &self,
            _opponent: &MatchmakingUser,
        ) -> Result<bool, crate::repositories::matchmaking_repository::MatchmakingRepositoryError>
        {
            Ok(self.reservation_succeeds)
        }
    }
}
