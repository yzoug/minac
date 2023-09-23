use std::io;

pub(crate) fn ask_for_move() -> String {
    println!("\nYour turn. Enter the SAN move. Example: Nf3");
    println!("To offer a draw, enter your next move and DRAW at the end. Example: Qxf5DRAW");
    println!("To resign, enter RESIGN");
    print!(">>> ");
    let mut next_move = String::new();
    io::stdin()
        .read_line(&mut next_move)
        .expect("IO Eroor: failed to read line");

    next_move.trim().to_string()
}

pub(crate) fn get_game_mode() -> u8 {

    println!("minac - minac Is Not A Chessboard
-----------------------
All moves must be in SAN (Standard Algebraic Notation).

Choose either offline (0) or online (1) game:
> ");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("IO Eroor: failed to read line.");

    choice.trim().parse().expect("Please type a number.")
}