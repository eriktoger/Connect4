use crate::structs::{MainState, UserAuth};
use common::{Game, Move, NewPlayer};
use rocket::State;
use uuid::Uuid;

#[get("/games")]
pub async fn get_games(main_state: &State<MainState>) -> String {
    let k: Vec<Game> = main_state.db.get_all_games().await;
    serde_json::to_string(&k).unwrap()
}
#[options("/games")]
pub fn options_game() {}

#[post("/games", data = "<player_1>")]
pub async fn create_game(
    main_state: &State<MainState>,
    player_1: String,
    _user_auth: UserAuth,
) -> String {
    let avaiable_channel = main_state.db.get_available_channel().await;
    if avaiable_channel.is_none() {
        return "".to_string();
    }

    let ac = avaiable_channel.unwrap();
    main_state.db.update_one_channel(ac.id.clone(), true).await;
    let id = Uuid::new_v4().to_string();
    let new_game = Game {
        id: id.clone(),
        player_1: player_1.clone(),
        player_2: Default::default(),
        grid: Default::default(),
        turn: player_1,
        channel: ac.id,
    };
    let _ = main_state.db.create_game(new_game).await;
    let games = main_state.db.get_all_games().await;
    let _res = main_state.lobby_channel.send(games);
    id
}

#[put("/games", data = "<data>")]
pub async fn update_game(main_state: &State<MainState>, data: String) {
    let deserialized: Game = serde_json::from_str(&data).unwrap();
    main_state.db.update_one_game(deserialized).await;
}

#[post("/games/join", data = "<data>")]
pub async fn join_game(main_state: &State<MainState>, data: String) {
    let deserialized: NewPlayer = serde_json::from_str(&data).unwrap();
    let game = main_state
        .db
        .get_one_game(deserialized.game.clone())
        .await
        .unwrap();
    if game.player_2 != "" {
        return;
    }
    main_state
        .db
        .join_game(deserialized.game, deserialized.player)
        .await;
}

#[post("/games/move", data = "<data>")]
pub async fn play_move(main_state: &State<MainState>, data: &str) {
    let deserialized: Move = serde_json::from_str(&data).unwrap();
    let mut game = main_state
        .db
        .get_one_game(deserialized.game_id)
        .await
        .unwrap();

    let not_your_turn = deserialized.player_id != game.turn;
    let no_player_2 = game.player_2 == "";
    let column_full = game.grid[5][deserialized.column] != "".to_string();
    let invalid_move = not_your_turn || no_player_2 || column_full;
    println!("{} {} {}", not_your_turn, no_player_2, column_full);
    if invalid_move {
        return;
    }

    for r in (0..6).rev() {
        let grid = &game.grid;
        let row = &grid[r];
        let square = &row[deserialized.column];

        if *square == "".to_string() {
            game.grid[r][deserialized.column] = deserialized.player_id;
            break;
        }
    }

    // switch turns;
    if game.turn == game.player_1 {
        game.turn = game.player_2.clone();
    } else {
        game.turn = game.player_1.clone();
    }

    // should check if the game is over.
    let current_channel = game.channel.clone();
    main_state.db.update_one_game(game.clone()).await;
    let sender = main_state.game_channels.get(&current_channel).unwrap();
    let _ = sender.send(game);
    //needs to send that move has been made.
}
