use std::io::{stdin,stdout};
use std::io::Write;

pub(crate) fn ask_for_move() -> String {
    println!("Your turn. Enter the SAN move. Example: Nf3");
    print!(">>> ");
    let _ = stdout().flush();

    let mut next_move = String::new();
    stdin().read_line(&mut next_move)
        .expect("IO Eroor: failed to read line");

    next_move.trim().to_string()
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
