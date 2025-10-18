pub mod game_session;

pub use game_session::GameSession;

/// Extracts the rating from a queue_rating string (format: "queue_type#rating")
pub fn extract_rating_from_queue_rating(queue_rating: &str) -> i32 {
    queue_rating
        .split('#')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Extracts the queue type from a queue_rating string (format: "queue_type#rating")
pub fn extract_queue_type_from_queue_rating(queue_rating: &str) -> String {
    queue_rating
        .split('#')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rating_from_queue_rating() {
        assert_eq!(extract_rating_from_queue_rating("rapid#1400"), 1400);
        assert_eq!(extract_rating_from_queue_rating("blitz#1200"), 1200);
        assert_eq!(extract_rating_from_queue_rating("bullet#1600"), 1600);
        assert_eq!(extract_rating_from_queue_rating("invalid"), 0);
        assert_eq!(extract_rating_from_queue_rating("rapid#"), 0);
    }

    #[test]
    fn test_extract_queue_type_from_queue_rating() {
        assert_eq!(extract_queue_type_from_queue_rating("rapid#1400"), "rapid");
        assert_eq!(extract_queue_type_from_queue_rating("blitz#1200"), "blitz");
        assert_eq!(
            extract_queue_type_from_queue_rating("bullet#1600"),
            "bullet"
        );
        assert_eq!(extract_queue_type_from_queue_rating("invalid"), "invalid");
        assert_eq!(extract_queue_type_from_queue_rating("#1400"), "");
    }
}
