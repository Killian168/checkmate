use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Game {
    pub game_id: String,
    pub white_player_id: String,
    pub black_player_id: String,
    pub time_control: String,
    pub status: GameStatus,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GameStatus {
    Active,
    Completed,
    Abandoned,
}
