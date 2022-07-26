#[macro_use]
extern crate rocket;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::http::Header;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{Request, Response};
use rocket::{Shutdown, State};

pub struct CORS;

// have one lobby
//presist it to a data base
// array or hashmap or rooms.
struct Rooms {
    room_1: Sender<Message>,
    room_2: Sender<Message>,
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
        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            "http://localhost:8080",
        ));
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
#[get("/events/<room>")]
async fn events(mut end: Shutdown, rooms: &State<Rooms>, room: String) -> EventStream![] {
    let mut current_room = if room == "1" {
        &rooms.room_1
    } else {
        &rooms.room_2
    };
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameInfo {
    room: String,
    message: String,
}
/// Receive a message from a form submission and broadcast it to any receivers.
/// Need to send json, with room and coordinates
#[post("/message2", data = "<data>")]
fn post(data: &str, rooms: &State<Rooms>) {
    // A send 'fails' if there are no active subscribers. That's okay.
    let deserialized: GameInfo = serde_json::from_str(&data).unwrap();
    println!("{}{}{}", data, deserialized.room, deserialized.message);
    let mut current_room = if deserialized.room == "1" {
        &rooms.room_1
    } else {
        &rooms.room_2
    };
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

#[launch]
fn rocket() -> _ {
    let rooms = Rooms {
        room_1: channel::<Message>(1024).0,
        room_2: channel::<Message>(1024).0,
    };
    rocket::build()
        .attach(CORS)
        .manage(rooms)
        .mount("/", routes![post2, post, events, home])
}
