mod events;
mod games;
mod mongodb_repo;
mod structs;
use crate::games::{create_game, get_games, get_one_game, join_game, play_move, update_game};
use common::{Game, UserInfo};
use events::{game_events, lobby_events};
#[macro_use]
extern crate rocket;

use mongodb_repo::MongoRepo;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::tokio::sync::broadcast::channel;
use rocket::Config;
use rocket::State;
use rocket::{Request, Response};
use std::collections::HashMap;
use std::env;

use crate::structs::MainState;
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let value = if Config::default().profile == "debug" {
            "http://localhost:8080"
        } else {
            "https://connect4rust.netlify.app"
        };

        response.set_header(Header::new("Access-Control-Allow-Origin", value));
        response.set_header(Header::new("Access-Control-Allow-Headers", "x-api-key"));
    }
}

#[post("/login", data = "<data>")]
async fn login(main_state: &State<MainState>, data: &str) -> String {
    let deserialized: UserInfo = serde_json::from_str(&data).unwrap();
    let response = main_state.db.auth_user(deserialized).await;
    serde_json::to_string(&response).unwrap()
}

#[post("/signup", data = "<data>")]
async fn create_user(main_state: &State<MainState>, data: &str) -> String {
    let deserialized: UserInfo = serde_json::from_str(&data).unwrap();
    let response = main_state.db.create_user(deserialized).await;
    serde_json::to_string(&response).unwrap()
}

#[options("/<_..>")]
pub fn all_options() {}

#[catch(503)]
fn service_unavailable() -> String {
    "Something went wrong".to_string()
}

#[catch(404)]
fn not_found() -> String {
    "Not found".to_string()
}

#[launch]
async fn rocket() -> _ {
    let mut game_channels = HashMap::new();
    let db = MongoRepo::init().await;

    let channels = db.get_all_channels().await;
    for chan in channels {
        game_channels.insert(chan.id, channel::<Game>(1024).0);
    }

    let main_state = MainState {
        game_channels,
        lobby_channel: channel::<Vec<Game>>(1024).0,
        db,
    };

    let default_port = 8000;
    let port: u64 = env::var("PORT")
        .and_then(|port| Ok(port.parse::<u64>().unwrap_or(default_port)))
        .unwrap_or(default_port);

    let config = Config::figment().merge(("port", port));

    rocket::custom(config)
        .attach(CORS)
        .manage(main_state)
        .register("/", catchers![service_unavailable, not_found])
        .mount(
            "/",
            routes![
                game_events,
                lobby_events,
                play_move,
                create_game,
                join_game,
                update_game,
                get_games,
                get_one_game,
                login,
                create_user,
                all_options,
            ],
        )
}
