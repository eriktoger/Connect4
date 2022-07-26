mod board;
use crate::components::Container;
use crate::{api_handler::ApiHandler, hooks::use_api_handler};
use board::Board;
use common::Game;
use yew::{function_component, html, use_effect_with_deps, use_state, Properties};

const STYLE_PATH: &str = include_str!("style.css");

#[derive(Properties, PartialEq)]
pub struct GameRoomProps {
    pub game_id: String,
}

#[function_component(GameRoom)]
pub fn game_room(props: &GameRoomProps) -> Html {
    let api_handler = use_api_handler();

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
    let game_events = use_state(|| {
        let route = format!("{}{}", "/game-events/", props.game_id);
        Some(ApiHandler::get_event_listener(
            route,
            move |new_game: Game| {
                game_clone.set(new_game);
            },
        ))
    });

    let game_clone = game.clone();
    let game_id = props.game_id.clone();
    let username = api_handler.user_info.username.clone();

    // stop listening to game
    if game.status.starts_with("player_") {
        if (*game_events).is_some() {
            game_events.set(None);
        }
    }

    use_effect_with_deps(
        move |_| {
            let game_clone = game_clone.clone();
            let route = format!("{}{}", "/games/", game_id);
            let on_success = move |new_game: Game| {
                game_clone.set(new_game);
            };
            api_handler.auth_get(route, on_success, || ());
            || ()
        },
        (),
    );
    html! {
        <Container path={STYLE_PATH}>
                {if game.turn == username && game.status == "active" {
                    html!{<h3>{"It's your turn"}</h3>}
                }else{
                    html!{<h3>{"Waiting..."}</h3>}
                }}
                <Board game={(*game).clone()} player_id={username} />
                <h2>{"status: "} {(*game).status.clone()}</h2>
        </Container>
    }
}
