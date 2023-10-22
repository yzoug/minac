use std::io::{stdin,stdout};
use std::io::Write;
use crate::online::commands::MoveOption;

pub(crate) fn ask_for_move() -> (String, Option<MoveOption>) {
    println!("Your turn. Enter the SAN move. Example: Nf3");
    print!(">>> ");
    let _ = stdout().flush();

    let mut line_parsed = String::new();
    stdin().read_line(&mut line_parsed)
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

pub(crate) fn get_game_mode() -> u8 {

    println!("minac - minac Is Not A Chessboard
-----------------------
All moves must be in SAN (Standard Algebraic Notation).

Choose either offline (0) or online (1) game:
");
    print!(">>> ");
    let _ = stdout().flush();

    let mut choice = String::new();
    stdin().read_line(&mut choice)
        .expect("IO Eroor: failed to read line.");

    choice.trim().parse().expect("Please type a number.")
}
