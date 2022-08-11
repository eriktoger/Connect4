use crate::hooks::use_api_handler;
use common::{Empty, Game};
use serde::{Deserialize, Serialize};
use stylist::Style;
use yew::{function_component, html, Callback, Properties};

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub game: Game,
    pub player_id: String,
}

#[derive(Serialize, Deserialize)]
struct Move {
    game_id: String,
    column: usize,
    player_id: String,
}

#[function_component(Board)]
pub fn board(props: &BoardProps) -> Html {
    let style_sheet = Style::new(include_str!("style.css")).expect("Css failed to load!");
    let first_row = props.game.grid.len();
    let items = (0..=first_row).collect::<Vec<_>>();
    let player_id = props.player_id.clone();
    let game_id = props.game.id.clone();
    let disabled = props.game.status != "active" || props.game.turn != player_id;
    html! {
    <div class={style_sheet} >
        <div class="drop-row">
            {for items.iter().enumerate().map(move |(i,_)|{
                html!{<DropToken column={i} player_id={props.player_id.clone()} game_id={game_id.clone()} {disabled}/>}
            })}
        </div>
        <div class="board">
            {for props.game.grid.iter().map(move |row|{
                let player_id = player_id.clone();
                html!{for row.iter().map(move |square| {
                        let empty ="".to_string();
                        let token = if *square == empty {""}  else if *square == player_id {"x"} else{"o"};
                        html!{<div class="square"> {token}</div>}
                    })}}

            )}
        </div>
    </div>
    }
}

#[derive(Properties, PartialEq)]
struct DropTokenProps {
    column: usize,
    player_id: String,
    game_id: String,
    disabled: bool,
}

#[function_component(DropToken)]
fn dropToken(props: &DropTokenProps) -> Html {
    let api_handler = use_api_handler();
    let id = props.game_id.clone();
    let player_id = props.player_id.clone();
    let column = props.column.clone();

    let drop_token = Callback::from(move |_| {
        let new_move = Move {
            column,
            game_id: id.clone(),
            player_id: player_id.clone(),
        };

        let route = "/games/move".to_string();
        match serde_json::to_string(&new_move) {
            Ok(body) => api_handler.auth_post(route, body, |_: Empty| (), || ()),
            Err(_) => (),
        }
    });

    html! {<button class="drop" disabled={props.disabled} onclick={drop_token.clone()}>

    </button>}
}
