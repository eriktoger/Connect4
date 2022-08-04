mod board;
use board::Board;
use common::Game;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{Event, EventSource, MessageEvent, Request};
use yew::{function_component, html, use_context, use_effect_with_deps, use_state, Properties};

use crate::{api_handler::ApiHandler, constants::API_ROUTE};

#[derive(Properties, PartialEq)]
pub struct GameRoomProps {
    pub game_id: String,
}

#[function_component(GameRoom)]
pub fn game_room(props: &GameRoomProps) -> Html {
    let ctx = use_context::<ApiHandler>().expect("Api handler context missing");
    let url = format!("{}{}{}", API_ROUTE, "/game-events/", props.game_id);
    let es = use_state(|| EventSource::new(&url).unwrap());

    let game = use_state(|| Game {
        id: props.game_id.clone(),
        player_1: Default::default(),
        player_2: Default::default(),
        grid: Default::default(),
        channel: Default::default(),
        turn: Default::default(),
        status: Default::default(),
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

    let game_clone = game.clone();
    let game_id = props.game_id.clone();
    let username = ctx.user_info.username.clone();
    use_effect_with_deps(
        move |_| {
            let game_clone = game_clone.clone();
            let url = format!("{}{}", "/games/", game_id);
            let action = move |new_game: Game| {
                game_clone.set(new_game);
            };
            ctx.get(url, action);
            || ()
        },
        (),
    );
    html! {
       <div>
        <span>{"this is a game room nr: "}{props.game_id.clone()}</span>
        <Board game={(*game).clone()} player_id={username} />
        <h2>{"Game Status:"} {(*game).status.clone()}</h2>
       </div>
    }
}
