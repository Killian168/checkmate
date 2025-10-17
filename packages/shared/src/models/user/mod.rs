use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug)]
pub enum UserRepositoryError {
    NotFound,
    AlreadyExists,
    Serialization(String),
    DynamoDb(String),
}

impl std::fmt::Display for UserRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRepositoryError::NotFound => write!(f, "User not found"),
            UserRepositoryError::AlreadyExists => write!(f, "User already exists"),
            UserRepositoryError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            UserRepositoryError::DynamoDb(msg) => write!(f, "DynamoDB error: {}", msg),
        }
    }
}

impl std::error::Error for UserRepositoryError {}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), UserRepositoryError>;
    async fn get_user_by_id(&self, user_id: &str) -> Result<User, UserRepositoryError>;
    async fn get_user_by_email(&self, email: &str) -> Result<User, UserRepositoryError>;
    async fn update_user(&self, user: &User) -> Result<(), UserRepositoryError>;
    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepositoryError>;
    async fn email_exists(&self, email: &str) -> Result<bool, UserRepositoryError>;
}
