use std::env;
extern crate dotenv;
use crate::game_model::Channel;
use common::{Game, UserInfo};
use dotenv::dotenv;
use mongodb::{
    bson::{doc, Bson},
    results::InsertOneResult,
    Client, Collection,
};
use rocket::futures::StreamExt;
use uuid::Uuid;

pub struct MongoRepo {
    game_col: Collection<Game>,
    channel_col: Collection<Channel>,
    user_col: Collection<UserInfo>,
}

//init
impl MongoRepo {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        };
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("Connect4");
        let game_col = db.collection("games");
        let channel_col = db.collection("channels");
        let user_col: Collection<UserInfo> = db.collection("users");

        //maybe should be its own function called reset or something.
        let _ = channel_col.delete_many(doc! {}, None).await;
        let _ = game_col.delete_many(doc! {}, None).await;
        let _ = user_col.delete_many(doc! {}, None).await;

        let mut docs = vec![];
        for _ in 0..3 {
            docs.push(Channel {
                _id: None,
                id: Uuid::new_v4().to_string(),
                taken: false,
            });
        }
        let _ = channel_col.insert_many(docs, None).await;

        let username = "admin".to_string();
        let password = "admin".to_string();
        let admin = UserInfo { username, password };
        let _ = user_col.insert_one(admin, None).await;

        MongoRepo {
            game_col,
            channel_col,
            user_col,
        }
    }

    pub async fn auth_user(&self, user: UserInfo) -> Option<String> {
        println!("{}{}", user.username, user.password);
        let filter = doc! {"username": user.username, "password":user.password};
        let user: Option<UserInfo> = self
            .user_col
            .find_one(filter.clone(), None)
            .await
            .ok()
            .unwrap();

        match user {
            Some(val) => {
                let api_key = Uuid::new_v4().to_string();
                let update = doc! { "$set": {"api_key": api_key.clone()}};
                let _ = self.user_col.update_one(filter, update, None).await;
                Some(api_key)
            }
            None => None,
        }
    }
}

// games
impl MongoRepo {
    pub async fn create_game(&self, new_game: Game) -> InsertOneResult {
        let channel = self.get_available_channel().await.unwrap().id;

        let new_doc = Game {
            id: new_game.id,
            player_1: new_game.player_1,
            player_2: new_game.player_2,
            grid: new_game.grid,
            turn: new_game.turn,
            channel,
        };

        self.game_col
            .insert_one(new_doc, None)
            .await
            .expect("Error creating user")
    }

    pub async fn get_one_game(&self, id: String) -> Option<Game> {
        let filter = doc! {"id": id};
        let game = self
            .game_col
            .find_one(filter, None)
            .await
            .ok()
            .expect("Error getting game");
        game
    }

    pub async fn get_all_games(&self) -> Vec<Game> {
        let mut game_cursor = self
            .game_col
            .find(None, None)
            .await
            .ok()
            .expect("Error getting all games");
        let mut result: Vec<Game> = Vec::new();
        while let Some(doc) = game_cursor.next().await {
            result.push(doc.unwrap());
        }
        result
    }

    pub async fn join_game(&self, id: String, player_2: String) {
        let query = doc! {"id": id.clone()};
        let update = doc! { "$set": {"player_2": player_2.clone()}};
        println!("{} {}", id, player_2);
        self.game_col
            .update_one(query, update, None)
            .await
            .expect("Error joining game");
    }

    pub async fn update_one_game(&self, replacement: Game) {
        let filter = doc! {"id": replacement.id.clone()};
        println!("222 {}", replacement.turn);
        self.game_col
            .replace_one(filter, replacement, None)
            .await
            .ok()
            .expect("Error getting channel");
    }
}

//Channels
impl MongoRepo {
    pub async fn get_all_channels(&self) -> Vec<Channel> {
        let mut cursor = self
            .channel_col
            .find(None, None)
            .await
            .ok()
            .expect("Error getting channels");
        let mut result: Vec<Channel> = Vec::new();
        while let Some(doc) = cursor.next().await {
            result.push(doc.unwrap());
        }
        result
    }

    pub async fn get_available_channel(&self) -> Option<Channel> {
        let filter = doc! {"taken": false};
        self.channel_col
            .find_one(filter, None)
            .await
            .ok()
            .expect("Error getting channel")
    }

    pub async fn update_one_channel(&self, id: Bson, taken: bool) {
        let filter = doc! {"_id": id};
        let update = doc! { "taken": taken };
        self.channel_col
            .update_one(filter, update, None)
            .await
            .ok()
            .expect("Error getting channel");
    }
}
