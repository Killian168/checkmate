use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub rating: i32,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, password: String, first_name: String, last_name: String) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            email,
            password,
            first_name,
            last_name,
            rating: 1200, // Default starting rating for chess platform
            created_at: Utc::now(),
        }
    }
}
