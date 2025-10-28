// Load .env file before running tests
fn load_env() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Try to load .env file from tests directory
        let _ = dotenv::from_filename("tests/.env");
        // Also try from current directory
        let _ = dotenv::dotenv();
    });
}

pub mod cognito_auth;
pub mod health_e2e;
