use serde::{Deserialize, Serialize};

pub const MAX_NUMBER_OF_GAMES: usize = 10;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewPlayer {
    pub game: String,
    pub player: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub player_1: String,
    pub player_2: String,
    pub grid: [[String; 7]; 6],
    pub turn: String,
    pub channel: String,
    pub status: String,
}
impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.turn == other.turn && self.status == other.status
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub password: String,
    pub api_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Move {
    pub game_id: String,
    pub column: usize,
    pub player_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameId {
    pub game_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Empty {}
