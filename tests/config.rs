use std::env;
use std::time::Duration;

/// Test configuration for the entire test suite
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Base URL for API endpoints
    pub base_url: String,
    /// JWT secret for testing
    pub jwt_secret: String,
    /// Test timeout duration
    pub test_timeout: Duration,
    /// Maximum concurrent test users
    pub max_concurrent_users: usize,
    /// Database table names for testing
    pub users_table: String,
    pub queue_table: String,
    pub game_sessions_table: String,
    pub player_connections_table: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            jwt_secret: "test-secret-key-for-jwt-tokens-in-testing".to_string(),
            test_timeout: Duration::from_secs(30),
            max_concurrent_users: 10,
            users_table: "test-users-table".to_string(),
            queue_table: "test-queue-table".to_string(),
            game_sessions_table: "test-game-sessions-table".to_string(),
            player_connections_table: "test-player-connections-table".to_string(),
        }
    }
}

impl TestConfig {
    /// Create a test configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(base_url) = env::var("TEST_BASE_URL") {
            config.base_url = base_url;
        }

        if let Ok(jwt_secret) = env::var("TEST_JWT_SECRET") {
            config.jwt_secret = jwt_secret;
        }

        if let Ok(timeout_secs) = env::var("TEST_TIMEOUT_SECONDS") {
            if let Ok(secs) = timeout_secs.parse() {
                config.test_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(max_users) = env::var("TEST_MAX_CONCURRENT_USERS") {
            if let Ok(users) = max_users.parse() {
                config.max_concurrent_users = users;
            }
        }

        if let Ok(users_table) = env::var("TEST_USERS_TABLE") {
            config.users_table = users_table;
        }

        if let Ok(queue_table) = env::var("TEST_QUEUE_TABLE") {
            config.queue_table = queue_table;
        }

        if let Ok(game_sessions_table) = env::var("TEST_GAME_SESSIONS_TABLE") {
            config.game_sessions_table = game_sessions_table;
        }

        if let Ok(player_connections_table) = env::var("TEST_PLAYER_CONNECTIONS_TABLE") {
            config.player_connections_table = player_connections_table;
        }

        config
    }

    /// Set up test environment variables
    pub fn setup_test_env(&self) {
        env::set_var("JWT_SECRET", &self.jwt_secret);
        env::set_var("USERS_TABLE", &self.users_table);
        env::set_var("QUEUE_TABLE", &self.queue_table);
        env::set_var("GAME_SESSIONS_TABLE", &self.game_sessions_table);
        env::set_var("PLAYER_CONNECTIONS_TABLE", &self.player_connections_table);
    }

    /// Clean up test environment variables
    pub fn cleanup_test_env() {
        env::remove_var("JWT_SECRET");
        env::remove_var("USERS_TABLE");
        env::remove_var("QUEUE_TABLE");
        env::remove_var("GAME_SESSIONS_TABLE");
        env::remove_var("PLAYER_CONNECTIONS_TABLE");
    }

    /// Check if we're running in CI environment
    pub fn is_ci_environment() -> bool {
        env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok()
    }

    /// Get test timeout with CI adjustment
    pub fn get_adjusted_timeout(&self) -> Duration {
        if Self::is_ci_environment() {
            // Increase timeout in CI environments
            self.test_timeout * 2
        } else {
            self.test_timeout
        }
    }
}

/// Test constants
pub mod constants {
    use std::time::Duration;

    /// Default test user credentials
    pub const DEFAULT_TEST_EMAIL: &str = "test@example.com";
    pub const DEFAULT_TEST_PASSWORD: &str = "test-password-123";
    pub const DEFAULT_TEST_FIRST_NAME: &str = "Test";
    pub const DEFAULT_TEST_LAST_NAME: &str = "User";

    /// Queue types for testing
    pub const QUEUE_TYPE_RAPID: &str = "rapid";
    pub const QUEUE_TYPE_BLITZ: &str = "blitz";
    pub const QUEUE_TYPE_BULLET: &str = "bullet";

    /// Test rating ranges
    pub const MIN_RATING: i32 = 0;
    pub const MAX_RATING: i32 = 3000;
    pub const DEFAULT_RATING: i32 = 1200;

    /// Test timeouts
    pub const SHORT_TIMEOUT: Duration = Duration::from_secs(5);
    pub const MEDIUM_TIMEOUT: Duration = Duration::from_secs(15);
    pub const LONG_TIMEOUT: Duration = Duration::from_secs(30);

    /// Test limits
    pub const MAX_TEST_USERS: usize = 100;
    pub const MAX_QUEUE_ENTRIES: usize = 50;
    pub const MAX_CONCURRENT_REQUESTS: usize = 10;
}

/// Test utilities
pub mod utils {
    use super::constants;
    use uuid::Uuid;

    /// Generate a unique test email
    pub fn generate_test_email() -> String {
        format!("test-{}@example.com", Uuid::new_v4())
    }

    /// Generate a unique test user ID
    pub fn generate_test_user_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate a test rating within valid range
    pub fn generate_test_rating(min: i32, max: i32) -> i32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }

    /// Validate email format for testing
    pub fn is_valid_test_email(email: &str) -> bool {
        email.contains('@') && email.ends_with(".com")
    }

    /// Validate password strength for testing
    pub fn is_valid_test_password(password: &str) -> bool {
        password.len() >= 8 && password.chars().any(|c| c.is_ascii_digit())
    }

    /// Generate a test queue rating string
    pub fn generate_queue_rating(queue_type: &str, rating: i32) -> String {
        let rating_bucket = (rating / 100) * 100;
        format!("{}#{}", queue_type, rating_bucket)
    }

    /// Parse queue rating string
    pub fn parse_queue_rating(queue_rating: &str) -> Option<(String, i32)> {
        let parts: Vec<&str> = queue_rating.split('#').collect();
        if parts.len() == 2 {
            if let Ok(rating) = parts[1].parse() {
                return Some((parts[0].to_string(), rating));
            }
        }
        None
    }
}
