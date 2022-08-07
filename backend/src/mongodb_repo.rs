use std::env;
extern crate dotenv;
use common::{Game, UserInfo};
use dotenv::dotenv;
use mongodb::bson::oid::ObjectId;
use mongodb::error::Error;
use mongodb::{bson::doc, results::InsertOneResult, Client, Collection};
use rocket::futures::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct MongoRepo {
    game_col: Collection<Game>,
    channel_col: Collection<Channel>,
    user_col: Collection<UserInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub id: String,
    pub taken: bool,
}

//init
impl MongoRepo {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = env::var("MONGO_URI").unwrap_or_default();
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("Connect4");
        let game_col = db.collection("games");
        let channel_col = db.collection("channels");
        let user_col: Collection<UserInfo> = db.collection("users");

        let db = MongoRepo {
            game_col,
            channel_col,
            user_col,
        };

        db.reset_data_base().await;
        db.add_admins().await;
        db.add_channels().await;
        db
    }

    async fn reset_data_base(&self) {
        let _ = self.channel_col.delete_many(doc! {}, None).await;
        let _ = self.game_col.delete_many(doc! {}, None).await;
        let _ = self.user_col.delete_many(doc! {}, None).await;
    }

    async fn add_admins(&self) {
        let admins = [("admin"), ("admin2")];

        for admin in admins {
            let username = admin.to_string();
            let password = admin.to_string();
            let api_key = Some(admin.to_string());
            let admin = UserInfo {
                username,
                password,
                api_key,
            };
            let _ = self.user_col.insert_one(admin, None).await;
        }
    }
    async fn add_channels(&self) {
        let mut docs = vec![];
        for _ in 0..3 {
            docs.push(Channel {
                _id: None,
                id: Uuid::new_v4().to_string(),
                taken: false,
            });
        }
        let _ = self.channel_col.insert_many(docs, None).await;
    }

    pub async fn auth_user(&self, user: UserInfo) -> Result<Option<String>, Error> {
        let filter = doc! {"username": user.username, "password":user.password};
        let result = self.user_col.find_one(filter.clone(), None).await;

        match result {
            Ok(option) => match option {
                Some(_) => {
                    let api_key = Uuid::new_v4().to_string();
                    let update = doc! { "$set": {"api_key": api_key.clone()}};
                    let _ = self.user_col.update_one(filter, update, None).await;
                    Ok(Some(api_key))
                }
                None => Ok(None),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn create_user(&self, user: UserInfo) -> Result<Option<String>, Error> {
        let filter = doc! {"username": user.username.clone()};
        let old_user = self.user_col.find_one(filter, None).await;

        if old_user.is_err() {
            return Err(old_user.err().unwrap());
        }

        let old_user = old_user.unwrap();

        if old_user.is_some() {
            return Ok(None);
        }

        let api_key = Some(Uuid::new_v4().to_string());
        let new_user = UserInfo {
            username: user.username,
            password: user.password,
            api_key: api_key.clone(),
        };
        let result = self.user_col.insert_one(new_user.clone(), None).await;
        match result {
            Ok(_) => Ok(api_key),
            Err(e) => Err(e),
        }
    }

    pub async fn user_is_auth(&self, username: String, api_key: String) -> Result<bool, Error> {
        let filter = doc! {"username": username, "api_key":api_key};
        let result = self.user_col.find_one(filter.clone(), None).await;

        match result {
            Ok(option) => match option {
                Some(_) => Ok(true),
                None => Ok(false),
            },
            Err(e) => Err(e),
        }
    }
}

// games
impl MongoRepo {
    pub async fn create_game(&self, new_game: Game) -> Result<InsertOneResult, Error> {
        self.game_col.insert_one(new_game, None).await
    }

    pub async fn get_one_game(&self, id: String) -> Result<Option<Game>, Error> {
        let filter = doc! {"id": id};
        self.game_col.find_one(filter, None).await
    }

    pub async fn get_all_games(&self) -> Result<Vec<Game>, Error> {
        let games = self.game_col.find(None, None).await;

        match games {
            Ok(mut cursor) => {
                let mut result: Vec<Game> = Vec::new();
                while let Some(doc) = cursor.next().await {
                    result.push(doc.unwrap());
                }
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn join_game(&self, id: String, player_2: String) {
        let query = doc! {"id": id.clone()};
        let update = doc! { "$set": {"player_2": player_2.clone(),"status":"active"}};
        self.game_col
            .update_one(query, update, None)
            .await
            .expect("Error joining game");
    }

    pub async fn update_one_game(&self, replacement: Game) {
        let filter = doc! {"id": replacement.id.clone()};
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

    pub async fn get_available_channel(&self) -> Result<Option<Channel>, Error> {
        let filter = doc! {"taken": false};
        self.channel_col.find_one(filter, None).await
    }

    pub async fn update_one_channel(&self, id: String, taken: bool) {
        println!("{id}");
        let filter = doc! {"id": id};
        let update = doc! { "$set": { "taken": taken }};
        self.channel_col
            .update_one(filter, update, None)
            .await
            .ok()
            .expect("Error updating channel");
    }
}
