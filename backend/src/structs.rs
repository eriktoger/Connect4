use crate::mongodb_repo::MongoRepo;
use common::Game;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::tokio::sync::broadcast::Sender;
use rocket::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct MainState {
    pub game_channels: HashMap<String, Sender<Game>>,
    pub lobby_channel: Sender<Vec<Game>>,
    pub db: MongoRepo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAuth {
    api_key: String,
    username: String,
}

#[derive(Debug)]
pub enum UserAuthError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth {
    type Error = UserAuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let main_state: &MainState = req.rocket().state().unwrap();

        match req.headers().get_one("x-api-key") {
            None => Outcome::Failure((Status::BadRequest, UserAuthError::Missing)),
            Some(key) => {
                let deserialized: UserAuth = serde_json::from_str(&key).unwrap();
                let is_authed = main_state
                    .db
                    .user_is_auth(deserialized.username.clone(), deserialized.api_key.clone())
                    .await;
                match is_authed {
                    true => Outcome::Success(deserialized),
                    false => Outcome::Failure((Status::BadRequest, UserAuthError::Invalid)),
                }
            }
        }
    }
}
