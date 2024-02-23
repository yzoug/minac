use chess::Color;
use std::io::Write;
use std::io::{stdin, stdout};

use crate::online::commands::MoveOption;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub lichess_study_id: String,
    pub lichess_token: String,
    pub stockfish_bin_path: String,
    pub stockfish_level: String,
}

pub(crate) fn ask_for_move() -> (String, Option<MoveOption>) {
    println!("Your turn. Enter the SAN move. Example: Nf3");
    print!(">>> ");
    let _ = stdout().flush();

    let mut line_parsed = String::new();
    stdin()
        .read_line(&mut line_parsed)
        .expect("IO Eroor: failed to read line");

    let command = line_parsed.trim();
    let mut option = None;

    if command.ends_with("DRAW") {
        option = Some(MoveOption::Draw);
    } else if command == "RESIGN" {
        option = Some(MoveOption::Resign);
    }

    (command.replace("DRAW", "").to_string(), option)
}

pub(crate) fn ask_for_side() -> Color {
    let returned_color;

    // keep asking if the input is wrong
    loop {
        println!("Choose which side you want to play: W for white, B for black.");
        print!(">>> ");
        let _ = stdout().flush();

        let mut line_parsed = String::new();
        stdin()
            .read_line(&mut line_parsed)
            .expect("IO Eroor: failed to read line");

        let command = line_parsed.trim();

        match command {
            "W" => returned_color = Color::White,
            "B" => returned_color = Color::Black,
            _ => {
                println!("Wrong input, try again.");
                continue;
            }
        };
        break;
    }
    returned_color
}

pub(crate) fn get_game_mode() -> u8 {
    println!(
        "minac - minac Is Not A Chessboard
-----------------------
All moves must be in SAN (Standard Algebraic Notation).

Choose either:
* [0] offline, 2 players
* [1] offline against stockfish
* [2] online
"
    );
    print!(">>> ");
    let _ = stdout().flush();

    let mut choice = String::new();
    stdin()
        .read_line(&mut choice)
        .expect("IO Eroor: failed to read line.");

    choice.trim().parse().expect("Please type a number.")
}
