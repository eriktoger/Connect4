use crate::{api_handler::ApiHandler, hooks::use_api_handler, routes::Route};
use common::{Empty, Game, GameId, NewPlayer, MAX_NUMBER_OF_GAMES};
use stylist::Style;
use web_sys::MouseEvent;
use yew::{function_component, html, use_effect_with_deps, use_state, Callback, Properties};
use yew_router::{history::AnyHistory, history::History, hooks::use_history};

#[function_component(Lobby)]
pub fn lobby() -> Html {
    let style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load");
    let games = use_state(|| vec![]);
    let api_handler = use_api_handler();
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
            let url = "/games".to_string();
            ApiHandler::get(url, on_success, || ());
            || ()
        },
        (),
    );

    let history = use_history().unwrap();
    let history_clone = history.clone();

    let disable_create_game = MAX_NUMBER_OF_GAMES <= (*games).len();

    html! {
        <main class={style_sheet}>
            <div class="container">
                <div>
                    <h1>{"Game Lobby"}</h1>

                    { if disable_create_game { html! {<h3>{"Maximum number of games reached"}</h3>}}else{
                        html!{<button disabled={disable_create_game} class="create-game" onclick={get_create_game( history_clone,api_handler.clone())}>{"Create new game"}</button>}
                    }}
                    <h2>{"Ongoing Games"}</h2>
                    <div class="game-container">
                        {if (*games).len() == 0{
                            html!{<p>{"No games available"}</p>}
                        } else {
                            html!{<GameList games={(*games).clone()}/>}
                        }}
                    </div>
                </div>
            </div>
        </main>
    }
}

#[derive(Properties, PartialEq)]
pub struct GameListProps {
    pub games: Vec<Game>,
}

#[function_component(GameList)]
pub fn game_list(props: &GameListProps) -> Html {
    let history = use_history().unwrap();
    let api_handler = use_api_handler();
    html! { for (props.games).iter().enumerate().map(move |(i,game)|{
         let create_game = get_join_game(game.clone(),history.clone(),api_handler.clone());
        html!{
            <div class="game-card" onclick={create_game}>
                <h3>{"Game: "}{i+1}</h3>
                <p>{"Player 1:"}{&game.player_1}</p>
                <p>{"Player 2:"}{&game.player_2}</p>
            </div>
        }}
    )}
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

        let on_success = move |_: Empty| {
            history.push(Route::Room {
                game_id: game.id.clone(),
            })
        };

        match serde_json::to_string(&new_player) {
            Ok(body) => api_handler.auth_post(route, body, on_success, || ()),
            Err(_) => (),
        }
    })
}

fn get_create_game(history: AnyHistory, api_handler: ApiHandler) -> yew::Callback<MouseEvent> {
    Callback::from(move |_| {
        let user_name = api_handler.user_info.username.clone();
        let history = history.clone();

        let route = "/games".to_string();
        let serialized = user_name.to_string();
        let on_success = move |response: GameId| {
            history.push(Route::Room {
                game_id: response.game_id,
            })
        };

        api_handler.auth_post(route, serialized, on_success, || ());
    })
}
