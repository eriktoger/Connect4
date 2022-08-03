mod board;
use board::Board;
use common::Game;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{Event, EventSource, MessageEvent};
use yew::{function_component, html, use_context, use_state, Properties};

use crate::{api_handler::ApiHandler, constants::API_ROUTE};

#[derive(Properties, PartialEq)]
pub struct GameRoomProps {
    pub game_id: String,
}

#[function_component(GameRoom)]
pub fn game_room(props: &GameRoomProps) -> Html {
    let url = format!("{}{}{}", API_ROUTE, "/game-events/", props.game_id);
    let ctx = use_context::<ApiHandler>().expect("Api handler context missing");
    let es = use_state(|| EventSource::new(&url).unwrap());

    let game = use_state(|| Game {
        id: props.game_id.clone(),
        player_1: Default::default(),
        player_2: Default::default(),
        grid: Default::default(),
        channel: Default::default(),
        turn: Default::default(),
    });

    let game_clone = game.clone();
    let _ = use_state(|| {
        EventListener::new(&es, "message", move |event: &Event| {
            let e = event.dyn_ref::<MessageEvent>().unwrap();
            let text = e.data().as_string().unwrap();
            let deserialized: Game = serde_json::from_str(&text).unwrap();
            game_clone.set(deserialized);
        })
    });

    html! {
       <div>
        <span>{"this is a game room nr: "}{props.game_id.clone()}</span>
        <Board game={(*game).clone()} player_id={ctx.user_info.username} />
       </div>
    }
}
