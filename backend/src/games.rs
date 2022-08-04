use crate::structs::{MainState, UserAuth};
use common::{Game, Move, NewPlayer};
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
pub async fn get_games(main_state: &State<MainState>) -> String {
    let k: Vec<Game> = main_state.db.get_all_games().await;
    serde_json::to_string(&k).unwrap()
}

#[get("/games/<game_id>")]
pub async fn get_one_game(main_state: &State<MainState>, game_id: String) -> String {
    let k: Game = main_state.db.get_one_game(game_id).await.unwrap();
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
        status: "not_started".to_string(),
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

    let game = main_state.db.get_one_game(game.id).await.unwrap();

    let sender = main_state.game_channels.get(&game.channel).unwrap();
    let _ = sender.send(game);
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
    let not_active = game.status != "active";
    let column_full = game.grid[0][deserialized.column] != "".to_string();
    let invalid_move = not_your_turn || not_active || column_full;

    if invalid_move {
        return;
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

    let current_channel = game.channel.clone();
    main_state.db.update_one_game(game.clone()).await;
    let sender = main_state.game_channels.get(&current_channel).unwrap();
    let _ = sender.send(game);
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
    let w = win.load(Ordering::Relaxed);
    println!("hej {}", w);
    w
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
    println!("horizontal");
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
