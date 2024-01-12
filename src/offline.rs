use chess::{ChessMove, Game};
use crate::online::commands::StockfishMode;
use crate::utils::{ask_for_move, ask_for_side};
use crate::stockfish::{launch_stockfish, send_move_to_stockfish, receive_stockfish_best_move};

pub(crate) fn offline_game_2_players() {
    let mut game = Game::new();
    println!("Offline game two players. You input all moves.");

    // while the game is still ongoing
    while game.result().is_none() {
        let current_board = game.current_position();
        println!("Current FEN: {}", current_board);

        println!("{:?} to move.", game.side_to_move());
        let (next_move_str, move_option) = ask_for_move();
        match move_option {
            Some(_) => {
                println!("Move option specified, ending game.");
                break;
            },
            _ => ()
        };

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

pub(crate) async fn offline_game_stockfish() {
    // play an offline game against stockfish
    let mut game = Game::new();
    let chosen_side = ask_for_side();
    debug!("Choosing side {:?}", chosen_side);

    // for now, we choose level 5 always when playing against stockfish
    let stockfish_level: u8 = 5;
    let stockfish_mode = StockfishMode::Offline {
        player_side: chosen_side,
        level: stockfish_level
    };

    // launch stockfish in a seperate thread
    let mut stockfish_child = launch_stockfish(stockfish_mode).await;

    // take the stdin and stdout of the stockfish child process
    let stockfish_in = stockfish_child.stdin.take().expect("Stockfish child has no stdin handle");
    let stockfish_out = stockfish_child.stdout.take().expect("Stockfish child has no stdin handle");

    // for now we test out the stdin/stdout
    // later, will need a task to handle the stdin stdout and a channel for communication
    send_move_to_stockfish(stockfish_in).await;
    receive_stockfish_best_move(stockfish_out).await;

    // while the game is still ongoing
    while game.result().is_none() {
        let current_board = game.current_position();
        println!("Current FEN: {}. {:?} to move.", current_board, game.side_to_move());

        if chosen_side == game.side_to_move() {
            // our turn
            let (next_move_str, move_option) = ask_for_move();
            match move_option {
                Some(_) => {
                    println!("Move option specified, ending game.");
                    break;
                },
                _ => ()
            };

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

            // send the move to stockfish
            // TODO

        } else {
            // stockfish's turn
            // parse what stockfish says
            // TODO
        }
    }

    println!("The game is over. Complete PGN for analysis: {}", game);

}