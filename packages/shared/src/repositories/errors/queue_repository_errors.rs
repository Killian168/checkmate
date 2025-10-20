#[derive(Debug)]
pub enum QueueRepositoryError {
    NotFound,
    AlreadyExists,
    Serialization(String),
    DynamoDb(String),
}

impl std::fmt::Display for QueueRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueRepositoryError::NotFound => write!(f, "MatchmakingUser not found"),
            QueueRepositoryError::AlreadyExists => {
                write!(f, "MatchmakingUser already exists")
            }
            QueueRepositoryError::Serialization(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            QueueRepositoryError::DynamoDb(msg) => write!(f, "DynamoDB error: {}", msg),
        }
    }
}

impl std::error::Error for QueueRepositoryError {}
