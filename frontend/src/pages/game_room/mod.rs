mod board;
use crate::api_handler::ApiHandler;
use board::Board;
use common::Game;
use yew::{function_component, html, use_context, use_effect_with_deps, use_state, Properties};

#[derive(Properties, PartialEq)]
pub struct GameRoomProps {
    pub game_id: String,
}

#[function_component(GameRoom)]
pub fn game_room(props: &GameRoomProps) -> Html {
    let api_handler = use_context::<ApiHandler>().expect("Api handler context missing");

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
    use_state(|| {
        let route = format!("{}{}", "/game-events/", props.game_id);
        ApiHandler::get_event_listener(route, move |new_game: Game| {
            game_clone.set(new_game);
        })
    });

    let game_clone = game.clone();
    let game_id = props.game_id.clone();
    let username = api_handler.user_info.username.clone();
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
       <div>
        <span>{"this is a game room nr: "}{props.game_id.clone()}</span>
        <Board game={(*game).clone()} player_id={username} />
        <h2>{"Game Status:"} {(*game).status.clone()}</h2>
       </div>
    }
}
