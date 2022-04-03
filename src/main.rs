use chess::{ChessMove, Game, Piece};
use std::io;

fn main() {
    match greetings() {
        0 => offline_game(),
        1 => online_game(),
        _ => {
            println!("Option not supported!");
            return;
        }
    };
}

fn offline_game() {
    let mut game = Game::new();

    // while the game is still ongoing
    while game.result().is_none() {
        let current_board = game.current_position();
        println!("Current FEN: {}", current_board);
        // DEBUG
        // display "combined" bitboard
        println!("DEBUG: current combined bitboard {:?}", *current_board.combined());
        println!("DEBUG: current bitboard for rooks {:?}", *current_board.pieces(Piece::Rook));

        println!("{:?} to move. Enter the SAN move (ex: Nf3):", game.side_to_move());

        let mut next_move_str = String::new();
        io::stdin()
            .read_line(&mut next_move_str)
            .expect("IO Eroor: failed to read line");

        // convert the SAN to a valid move
        // will repeat the loop if the move is not valid
        let next_move = match ChessMove::from_san(&current_board, &next_move_str) {
            Ok(m) => m,
            Err(e) => {
                println!("Problem with the chess move, try again. Error: {}", e);
                continue;
            },
        };

        // make the move
        game.make_move(next_move);

    }

    println!("The game is over. Complete PGN for analysis: {}", game);

}

fn online_game() {
    
}


fn greetings() -> u8 {

    println!("MINAC - MINAC Is Not A Chessboard
    -----------------------

    v0.0.1a

    All moves must be in SAN (Standard Algebraic Notation).

    Choose either offline (0) or online (1) game:
    ");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("IO Eroor: failed to read line");

    choice.trim().parse().expect("Please type a number")

}
