use crate::pages::game_room::GameRoom;

use crate::pages::lobby::Lobby;

use yew::{html, Html};
use yew_router::Routable;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/lobby")]
    Lobby,
    #[at("/room/:game_id")]
    Room { game_id: String },
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::Lobby => html! {<Lobby/>},
        Route::Room { game_id } => {
            html! {<GameRoom game_id={game_id.clone()} />}
        }
    }
}
