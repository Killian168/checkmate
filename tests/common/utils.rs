use rand::{distributions::Alphanumeric, Rng};
use std::env;
use uuid::Uuid;

pub fn base_url() -> String {
    env::var("BASE_URL").expect("Missing BASE_URL environment variable")
}

pub fn test_password() -> String {
    env::var("TEST_PASSWORD").unwrap_or_else(|_| random_string(16))
}

pub fn random_email() -> String {
    format!("it_{}@example.com", Uuid::new_v4())
}

pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
