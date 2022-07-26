use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameRoom {
    pub id: String,
    pub player_1: String,
    pub player_2: String,
    pub turn: String,
    pub moves: Vec<String>,
    pub chat_log: Vec<String>,
    pub grid: [[String; 7]; 6],
}
