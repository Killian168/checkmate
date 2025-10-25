use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRequest {
    pub game_session_id: String,
    pub from_square: String,             // e.g., "e2"
    pub to_square: String,               // e.g., "e4"
    pub promotion_piece: Option<String>, // e.g., "q" for queen
}

impl MoveRequest {
    pub fn new(game_session_id: String, from_square: String, to_square: String) -> Self {
        MoveRequest {
            game_session_id,
            from_square,
            to_square,
            promotion_piece: None,
        }
    }

    pub fn with_promotion(
        game_session_id: String,
        from_square: String,
        to_square: String,
        promotion_piece: String,
    ) -> Self {
        MoveRequest {
            game_session_id,
            from_square,
            to_square,
            promotion_piece: Some(promotion_piece),
        }
    }
}
