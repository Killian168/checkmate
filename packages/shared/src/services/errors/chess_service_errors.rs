use std::fmt;

#[derive(Debug)]
pub enum ChessServiceError {
    ValidationError(String),
    IllegalMove(String),
    GameOver(String),
    InvalidPosition(String),
}

impl fmt::Display for ChessServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChessServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ChessServiceError::IllegalMove(msg) => write!(f, "Illegal move: {}", msg),
            ChessServiceError::GameOver(msg) => write!(f, "Game over: {}", msg),
            ChessServiceError::InvalidPosition(msg) => write!(f, "Invalid position: {}", msg),
        }
    }
}

impl std::error::Error for ChessServiceError {}
