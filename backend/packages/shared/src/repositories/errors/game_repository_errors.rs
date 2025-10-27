#[derive(Debug)]
pub enum GameSessionRepositoryError {
    Serialization(String),
    DynamoDb(String),
}

impl std::fmt::Display for GameSessionRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameSessionRepositoryError::Serialization(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            GameSessionRepositoryError::DynamoDb(msg) => write!(f, "DynamoDB error: {}", msg),
        }
    }
}

impl std::error::Error for GameSessionRepositoryError {}
