use crate::pages::game_room::GameRoom;
use crate::pages::home::Home;
use crate::pages::lobby::Lobby;

use yew::{html, Html};
use yew_router::Routable;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/lobby")]
    Lobby,
    #[at("/room/:game_id/:player_id")]
    Room { game_id: String, player_id: String },
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! {<Home/>},
        Route::Lobby => html! {<Lobby/>},
        Route::Room { game_id, player_id } => {
            html! {<GameRoom game_id={game_id.clone()} player_id={player_id.clone()} />}
        }
    }
}
