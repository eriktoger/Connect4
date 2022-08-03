use crate::{api_handler::ApiHandler, constants::API_ROUTE, routes::Route};
use common::{Game, NewPlayer};
use gloo_events::EventListener;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use stylist::Style;
use wasm_bindgen::JsCast;
use web_sys::{Event, EventSource, MessageEvent, MouseEvent};
use yew::{function_component, html, use_context, use_effect_with_deps, use_state, Callback};
use yew_router::{history::AnyHistory, history::History, hooks::use_history};

#[derive(Serialize, Deserialize)]
struct UserAuth {
    api_key: String,
    username: String,
}

#[function_component(Lobby)]
pub fn lobby() -> Html {
    let style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load");
    let games = use_state(|| vec![]);
    let ctx = use_context::<ApiHandler>().expect("Api handler context missing");

    let games_clone = games.clone();
    let url = format!("{}{}", API_ROUTE, "/lobby-events/");
    let es = use_state(|| EventSource::new(&url).unwrap());
    use_state(|| {
        EventListener::new(&es, "message", move |event: &Event| {
            let e = event.dyn_ref::<MessageEvent>().unwrap();
            let text = e.data().as_string().unwrap();
            let deserialized: Vec<Game> = serde_json::from_str(&text).unwrap();
            games_clone.set(deserialized);
        })
    });

    let games_clone = games.clone();
    use_effect_with_deps(
        |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("{}{}", API_ROUTE, "/games");

                let response: Vec<Game> = Request::get(&url)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                games_clone.set(response);
            });
            || ()
        },
        (),
    );

    let history = use_history().unwrap();
    let history_clone = history.clone();
    let username = ctx.user_info.username.clone();
    html! {
        <main class={style_sheet}>
            <div>{"Welcome to the lobby!"}</div>
            <div class="card-container">
                {for (*games).iter().map(move |game|{
                    let create_game = get_join_game(game.clone(),username.clone(),history.clone());
                    html!{
                        <div class="game-card" onclick={create_game}>
                            <h1>{"id:"}{&game.id}</h1>
                            <p>{"Player 1:"}{&game.player_1}</p>
                            <p>{"Player 2:"}{&game.player_2}</p>
                        </div>
                    }}
                )}
             </div>
             <button onclick={get_create_game( history_clone,ctx)}>{"Create new game"}</button>
       </main>
    }
}

fn get_join_game(game: Game, user_name: String, history: AnyHistory) -> yew::Callback<MouseEvent> {
    Callback::from(move |_| {
        let user_name = user_name.clone();
        let game = game.clone();
        let history = history.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let new_player = NewPlayer {
                player: user_name.to_string(),
                game: game.clone().id,
            };
            let serialized = serde_json::to_string(&new_player).unwrap();
            let url = format!("{}{}", API_ROUTE, "/games/join");
            let _ = Request::post(&url).body(&serialized).send().await;
            history.push(Route::Room { game_id: game.id });
        });
    })
}

fn get_create_game(history: AnyHistory, ctx: ApiHandler) -> yew::Callback<MouseEvent> {
    Callback::from(move |_| {
        let user_name = ctx.user_info.username.clone();
        let history = history.clone();
        let user_info = ctx.user_info.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}{}", API_ROUTE, "/games");
            let user_auth = UserAuth {
                username: user_info.username,
                api_key: user_info.api_key.unwrap_or_default(),
            };
            let serialized = serde_json::to_string(&user_auth).unwrap();
            let response: String = Request::post(&url)
                .body(user_name.to_string())
                .header("x-api-key", &serialized)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

            history.push(Route::Room { game_id: response });
        });
    })
}
