use serde::{Deserialize, Serialize};

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
}
impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.turn == other.turn
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub password: String,
}
