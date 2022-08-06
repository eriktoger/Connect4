use crate::{api_handler::ApiHandler, routes::Route};
use common::{Empty, Game, GameId, NewPlayer};
use stylist::Style;
use web_sys::MouseEvent;
use yew::{function_component, html, use_context, use_effect_with_deps, use_state, Callback};
use yew_router::{history::AnyHistory, history::History, hooks::use_history};

#[function_component(Lobby)]
pub fn lobby() -> Html {
    let style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load");
    let games = use_state(|| vec![]);
    let api_handler = use_context::<ApiHandler>().expect("Api handler context missing");
    let games_clone = games.clone();

    use_state(|| {
        let url = "/lobby-events/".to_string();
        let action = move |new_games: Vec<Game>| {
            games_clone.set(new_games);
        };
        ApiHandler::get_event_listener(url, action)
    });

    let games_clone = games.clone();

    use_effect_with_deps(
        move |_| {
            let on_success = move |new_games: Vec<Game>| {
                games_clone.set(new_games);
            };
            let url = "/games/".to_string();
            ApiHandler::get(url, on_success, || ());
            || ()
        },
        (),
    );

    let history = use_history().unwrap();
    let history_clone = history.clone();
    let api_handler_clone = api_handler.clone();
    html! {
        <main class={style_sheet}>
            <div>{"Welcome to the lobby!"}</div>
            <div class="card-container">
                {for (*games).iter().map(move |game|{
                    let create_game = get_join_game(game.clone(),history.clone(),api_handler_clone.clone());
                    html!{
                        <div class="game-card" onclick={create_game}>
                            <h1>{"id:"}{&game.id}</h1>
                            <p>{"Player 1:"}{&game.player_1}</p>
                            <p>{"Player 2:"}{&game.player_2}</p>
                        </div>
                    }}
                )}
             </div>
             <button onclick={get_create_game( history_clone,api_handler.clone())}>{"Create new game"}</button>
       </main>
    }
}

fn get_join_game(
    game: Game,
    history: AnyHistory,
    api_handler: ApiHandler,
) -> yew::Callback<MouseEvent> {
    Callback::from(move |_| {
        let game = game.clone();
        let history = history.clone();
        let route = "/games/join".to_string();
        let new_player = NewPlayer {
            player: api_handler.user_info.username.to_string(),
            game: game.clone().id,
        };
        let serialized = serde_json::to_string(&new_player).unwrap();

        let action = move |_: Empty| {
            history.push(Route::Room {
                game_id: game.id.clone(),
            })
        };
        api_handler.auth_post(route, serialized, action);
    })
}

fn get_create_game(history: AnyHistory, api_handler: ApiHandler) -> yew::Callback<MouseEvent> {
    Callback::from(move |_| {
        let user_name = api_handler.user_info.username.clone();
        let history = history.clone();

        let route = "/games".to_string();
        let serialized = user_name.to_string();
        let action = move |response: GameId| {
            history.push(Route::Room {
                game_id: response.game_id,
            })
        };
        api_handler.auth_post(route, serialized, action);
    })
}
