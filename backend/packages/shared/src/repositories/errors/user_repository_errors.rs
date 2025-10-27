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
