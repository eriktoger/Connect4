mod game_model;
mod game_room;
pub mod mongodb_repo;
use common::{Game, NewPlayer};
#[macro_use]
extern crate rocket;

use mongodb_repo::MongoRepo;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::Config;
use rocket::{Request, Response};
use rocket::{Shutdown, State};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;
pub struct CORS;

// have one lobby
//presist it to a data base
// array or hashmap or rooms.

struct MainState {
    game_channels: HashMap<String, Sender<Game>>,
    lobby_channel: Sender<Vec<Game>>,
    db: MongoRepo,
}

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        if Config::default().profile == "debug" {
            response.set_header(Header::new(
                "Access-Control-Allow-Origin",
                "http://127.0.0.1:8080",
            ));
        } else {
            response.set_header(Header::new(
                "Access-Control-Allow-Origin",
                "https://connect4rust.netlify.app",
            ));
        }
    }
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..20))]
    pub username: String,
    pub message: String,
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events/<game_id>")]
async fn events(
    mut end: Shutdown,
    main_state: &State<MainState>,
    game_id: String,
) -> EventStream![] {
    let game = main_state.db.get_one_game(game_id).await.unwrap();
    let current_room = main_state.game_channels.get(&game.channel).unwrap();
    let mut rx = current_room.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

           yield Event::json(&msg);
        }
    }
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/lobby-events")]
async fn lobby_events(mut end: Shutdown, main_state: &State<MainState>) -> EventStream![] {
    //lobby events should trigger when a new game is:
    // created, a player joins/leaves game, or game has ended
    let mut rx = main_state.lobby_channel.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

           yield Event::json(&msg);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameInfo {
    room: String,
    message: String,
}
/// Receive a message from a form submission and broadcast it to any receivers.
/// Need to send json, with room and coordinates
///
/*
#[post("/message2", data = "<data>")]
fn post(data: &str, main_state: &State<MainState>) {
    // A send 'fails' if there are no active subscribers. That's okay.
    let deserialized: GameInfo = serde_json::from_str(&data).unwrap();
    println!("{}{}{}", data, deserialized.room, deserialized.message);
    let current_room = main_state.game_channels.get("1").unwrap();
    let new_message = Message {
        room: deserialized.room.to_string(),

        username: "1".to_string(),
        message: deserialized.message.to_string(),
    };
    let _res = current_room.send(new_message);
}

#[post("/post2", data = "<data>")]
fn post2(data: String) -> String {
    println!("post2: {}", data);
    data
}

#[get("/home")]
fn home() -> String {
    "hej".to_string()
}
 */

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameRoom {
    id: String,
    player_1: String,
    player_2: String,
    turn: String,
    moves: Vec<String>,
    chat_log: Vec<String>,
    grid: [[String; 7]; 6],
}

#[get("/games")]
async fn get_games(main_state: &State<MainState>) -> String {
    let k: Vec<Game> = main_state.db.get_all_games().await;
    serde_json::to_string(&k).unwrap()
}

#[post("/games", data = "<player_1>")]
async fn create_game(main_state: &State<MainState>, player_1: String) -> String {
    let avaiable_channel = main_state.db.get_available_channel().await;
    if avaiable_channel.is_none() {
        return "".to_string();
    }
    let ac = avaiable_channel.unwrap();
    let id = Uuid::new_v4().to_string();
    let new_game = Game {
        id: id.clone(),
        player_1: player_1.clone(),
        player_2: Default::default(),
        grid: Default::default(),
        turn: player_1,
        channel: ac.id,
    };
    let _ = main_state.db.create_game(new_game).await;
    let games = main_state.db.get_all_games().await;
    let _res = main_state.lobby_channel.send(games);
    id
}

#[put("/games", data = "<data>")]
async fn update_game(main_state: &State<MainState>, data: String) {
    let deserialized: Game = serde_json::from_str(&data).unwrap();
    main_state.db.update_one_game(deserialized).await;
}

#[post("/games/join", data = "<data>")]
async fn join_game(main_state: &State<MainState>, data: String) {
    let deserialized: NewPlayer = serde_json::from_str(&data).unwrap();
    let game = main_state
        .db
        .get_one_game(deserialized.game.clone())
        .await
        .unwrap();
    if game.player_2 != "" {
        return;
    }
    main_state
        .db
        .join_game(deserialized.game, deserialized.player)
        .await;
}

#[derive(Serialize, Deserialize, Clone)]
struct Move {
    game_id: String,
    column: usize,
    player_id: String,
}

#[post("/move", data = "<data>")]
async fn play_move(main_state: &State<MainState>, data: &str) {
    println!("{}", data);

    let deserialized: Move = serde_json::from_str(&data).unwrap();
    let mut game = main_state
        .db
        .get_one_game(deserialized.game_id)
        .await
        .unwrap();

    if deserialized.player_id != game.turn || game.player_2 == "" {
        return;
    }

    for r in (0..6).rev() {
        let grid = &game.grid;
        let row = &grid[r];
        let square = &row[deserialized.column];

        if *square == "".to_string() {
            println!("hejsan hejsan");
            game.grid[r][deserialized.column] = deserialized.player_id;
            break;
        }
    }
    // switch turns;
    if game.turn == game.player_1 {
        game.turn = game.player_2.clone();
    } else {
        game.turn = game.player_1.clone();
    }

    // should check if the game is over.
    let current_channel = game.channel.clone();
    main_state.db.update_one_game(game.clone()).await;
    let sender = main_state.game_channels.get(&current_channel).unwrap();
    let _ = sender.send(game);
    //needs to send that move has been made.
}

#[launch]
async fn rocket() -> _ {
    // I need to move the game_rooms to a database
    //https://dev.to/hackmamba/build-a-rest-api-with-rust-and-mongodb-rocket-version-ah5

    let mut game_channels = HashMap::new();
    let db = MongoRepo::init().await;

    let channels = db.get_all_channels().await;
    for chan in channels {
        // maybe should be Bson and not string, lets see
        //Bson was a pain to work with
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
        .mount(
            "/",
            routes![
                events,
                lobby_events,
                play_move,
                create_game,
                join_game,
                update_game,
                get_games,
            ],
        )
}
