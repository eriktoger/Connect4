use crate::structs::{MainState, UserAuth};
use common::{Empty, Game, GameId, Move, NewPlayer};
use rocket::http::Status;
use rocket::State;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
use uuid::Uuid;

#[get("/games")]
pub async fn get_games(main_state: &State<MainState>) -> Result<String, Status> {
    match main_state.db.get_active_games().await {
        Ok(games) => Ok(serde_json::to_string(&games).unwrap()),
        Err(_) => Err(Status::ServiceUnavailable),
    }
}

#[get("/games/<game_id>")]
pub async fn get_one_game(
    main_state: &State<MainState>,
    game_id: String,
) -> Result<String, Status> {
    let game_result = main_state.db.get_one_game(game_id).await;
    match game_result {
        Ok(game_option) => match game_option {
            Some(game) => Ok(serde_json::to_string(&game).unwrap()),
            None => Err(Status::NotFound),
        },
        Err(_) => Err(Status::ServiceUnavailable),
    }
}

#[post("/games", data = "<player_1>")]
pub async fn create_game(
    main_state: &State<MainState>,
    player_1: String,
    _user_auth: UserAuth,
) -> Result<String, Status> {
    let result = main_state.db.get_available_channel().await;
    if result.is_err() {
        return Err(Status::ServiceUnavailable);
    }

    let option = result.unwrap();
    if option.is_none() {
        return Err(Status::Conflict);
    }

    let channel = option.unwrap();

    main_state
        .db
        .update_one_channel(channel.id.clone(), true)
        .await;
    let id = Uuid::new_v4().to_string();
    let new_game = Game {
        id: id.clone(),
        player_1: player_1.clone(),
        player_2: Default::default(),
        grid: Default::default(),
        turn: player_1,
        channel: channel.id,
        status: "not_started".to_string(),
    };
    let _ = main_state.db.create_game(new_game).await;

    match main_state.db.get_active_games().await {
        Ok(games) => {
            let _res = main_state.lobby_channel.send(games);
            Ok(serde_json::to_string(&GameId { game_id: id }).unwrap())
        }
        Err(_) => Err(Status::ServiceUnavailable),
    }
}

#[put("/games", data = "<data>")]
pub async fn update_game(main_state: &State<MainState>, data: String) {
    let deserialized: Game = serde_json::from_str(&data).unwrap();
    main_state.db.update_one_game(deserialized).await;
}

#[post("/games/join", data = "<data>")]
pub async fn join_game(
    main_state: &State<MainState>,
    data: String,
    _user_auth: UserAuth,
) -> Result<String, Status> {
    let deserialized: NewPlayer = serde_json::from_str(&data).unwrap();
    let game_result = main_state.db.get_one_game(deserialized.game.clone()).await;
    if game_result.is_err() {
        return Err(Status::ServiceUnavailable);
    }
    let game_option = game_result.unwrap();

    if game_option.is_none() {
        return Err(Status::NotFound);
    }

    let game = game_option.unwrap();

    if game.status != "not_started" {
        return Ok(serde_json::to_string(&Empty {}).unwrap());
    }

    let result = main_state
        .db
        .join_game(deserialized.game, deserialized.player)
        .await;

    if result.is_err() {
        return Err(Status::ServiceUnavailable);
    }

    let game_result = main_state.db.get_one_game(game.id).await;

    if game_result.is_err() {
        return Err(Status::ServiceUnavailable);
    }
    let game_option = game_result.unwrap();

    if game_option.is_none() {
        return Err(Status::NotFound);
    }

    let game = game_option.unwrap();

    let sender = main_state.game_channels.get(&game.channel);
    match sender {
        Some(sender) => {
            let _ = sender.send(game);
        }
        None => (),
    };

    Ok(serde_json::to_string(&Empty {}).unwrap())
}

#[post("/games/move", data = "<data>")]
pub async fn play_move(main_state: &State<MainState>, data: &str) -> Result<String, Status> {
    let deserialized: Move = serde_json::from_str(&data).unwrap();
    let game_result = main_state.db.get_one_game(deserialized.game_id).await;

    if game_result.is_err() {
        return Err(Status::ServiceUnavailable);
    }
    let game_option = game_result.unwrap();

    if game_option.is_none() {
        return Err(Status::NotFound);
    }

    let mut game = game_option.unwrap();

    let not_your_turn = deserialized.player_id != game.turn;
    let not_active = game.status != "active";
    let column_full = game.grid[0][deserialized.column] != "".to_string();
    let invalid_move = not_your_turn || not_active || column_full;

    let empty_return = serde_json::to_string(&Empty {}).unwrap();
    if invalid_move {
        return Ok(empty_return);
    }

    for row_index in (0..6).rev() {
        let grid = &game.grid;
        let row = &grid[row_index];
        let square = &row[deserialized.column];

        if *square == "".to_string() {
            let player_id = deserialized.player_id;
            let col_index = deserialized.column;

            if player_won(&player_id, grid, row_index, col_index) {
                game.status = if player_id == game.player_1 {
                    "player_1_won".to_string()
                } else {
                    "player_2_won".to_string()
                };
            }
            game.grid[row_index][deserialized.column] = player_id;

            break;
        }
    }

    // switch turns;
    if game.turn == game.player_1 {
        game.turn = game.player_2.clone();
    } else {
        game.turn = game.player_1.clone();
    }

    main_state.db.update_one_game(game.clone()).await;

    let sender = main_state.game_channels.get(&game.channel);
    match sender {
        Some(sender) => {
            let _ = sender.send(game.clone());
        }
        None => (),
    };

    if game.status == "player_1_won" || game.status == "player_2_won" {
        let channel = game.channel.clone();
        game.channel = "".to_string();
        main_state.db.update_one_channel(channel, false).await;
        match main_state.db.get_active_games().await {
            Ok(games) => {
                let _res = main_state.lobby_channel.send(games);
            }
            Err(_) => (),
        }
    }

    Ok(empty_return)
}

fn player_won(
    player_id: &String,
    grid: &[[String; 7]; 6],
    row_index: usize,
    col_index: usize,
) -> bool {
    let win = Arc::new(AtomicBool::new(false));
    let win_conditions: Vec<fn(&String, &[[String; 7]; 6], usize, usize) -> bool> = vec![
        horizontal_win,
        vertical_win,
        diagonally_win,
        reverse_diagnolly_win,
    ];

    for win_condition in win_conditions {
        let player_id = player_id.clone();
        let grid = grid.clone();
        let win = win.clone();
        thread::spawn(move || {
            if win_condition(&player_id, &grid, row_index, col_index) {
                win.store(true, Ordering::Relaxed);
            }
        });
    }
    win.load(Ordering::Relaxed)
}

fn horizontal_win(
    player_id: &String,
    grid: &[[String; 7]; 6],
    row_index: usize,
    col_index: usize,
) -> bool {
    let mut check_left = true;
    let mut check_right = true;
    let mut in_a_row = 1;

    for i in 1..4 {
        let in_bound = col_index >= i;
        if check_left && in_bound && grid[row_index][col_index - i] == *player_id {
            in_a_row += 1;
        } else {
            check_left = false;
        }

        let in_bound = col_index + i < 7;
        if check_right && in_bound && grid[row_index][col_index + i] == *player_id {
            in_a_row += 1;
        } else {
            check_right = false;
        }
    }
    in_a_row > 3
}

fn vertical_win(
    player_id: &String,
    grid: &[[String; 7]; 6],
    row_index: usize,
    col_index: usize,
) -> bool {
    let mut check_down = true;
    let mut in_a_row = 1;

    for i in 1..4 {
        let in_bound = row_index + i < 6;
        if check_down && in_bound && grid[row_index + i][col_index] == *player_id {
            in_a_row += 1;
        } else {
            check_down = false;
        }
    }
    in_a_row > 3
}

fn diagonally_win(
    player_id: &String,
    grid: &[[String; 7]; 6],
    row_index: usize,
    col_index: usize,
) -> bool {
    let mut check_up_left = true;
    let mut check_down_right = true;
    let mut in_a_row = 1;

    for i in 1..4 {
        let in_bound = row_index >= i && col_index >= i;
        if check_up_left && in_bound && grid[row_index - i][col_index - i] == *player_id {
            in_a_row += 1;
        } else {
            check_up_left = false;
        }

        let in_bound = row_index + i < 6 && col_index + i < 7;
        if check_down_right && in_bound && grid[row_index + i][col_index + i] == *player_id {
            in_a_row += 1;
        } else {
            check_down_right = false;
        }
    }
    in_a_row > 3
}

fn reverse_diagnolly_win(
    player_id: &String,
    grid: &[[String; 7]; 6],
    row_index: usize,
    col_index: usize,
) -> bool {
    let mut check_up_left = true;
    let mut check_down_right = true;
    let mut in_a_row = 1;

    for i in 1..4 {
        let in_bound = row_index + i < 6 && col_index >= i;

        if check_up_left && in_bound && grid[row_index + i][col_index - i] == *player_id {
            in_a_row += 1;
        } else {
            check_up_left = false;
        }

        let in_bound = row_index >= i && col_index + i < 7;
        if check_down_right && in_bound && grid[row_index - i][col_index + i] == *player_id {
            in_a_row += 1;
        } else {
            check_down_right = false;
        }
    }
    in_a_row > 3
}
