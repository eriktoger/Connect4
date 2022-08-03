extern crate rocket;
use crate::structs::MainState;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::error::RecvError;
use rocket::{Shutdown, State};

#[get("/game-events/<game_id>")]
pub async fn game_events(
    mut end: Shutdown,
    main_state: &State<MainState>,
    game_id: String,
) -> EventStream![] {
    let game = main_state.db.get_one_game(game_id).await.unwrap();
    let current_room = main_state.game_channels.get(&game.channel).unwrap();
    let mut rx = current_room.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

           yield Event::json(&msg);
        }
    }
}

#[get("/lobby-events")]
pub async fn lobby_events(mut end: Shutdown, main_state: &State<MainState>) -> EventStream![] {
    //lobby events should trigger when a new game is:
    // created, a player joins/leaves game, or game has ended
    let mut rx = main_state.lobby_channel.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

           yield Event::json(&msg);
        }
    }
}
