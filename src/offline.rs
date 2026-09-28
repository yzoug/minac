use crate::online::commands::{StockfishInput, StockfishOutput};
use crate::stockfish::{launch_stockfish, receive_stockfish_best_move, send_move_to_stockfish};
use crate::utils::ask_for_move;
use chess::{ChessMove, Color, Game};
use log::{debug, error};
use std::io::{BufRead, BufReader};
use tokio::spawn;
use tokio::sync::mpsc;

const DEPTH: i64 = 5;
const LEVEL: i64 = 1;

pub(crate) fn offline_game_2_players(input: &mut dyn BufRead) -> Game {
    let mut game = Game::new();
    println!("Offline game two players. You input all moves.");

    while game.result().is_none() {
        let current_board = game.current_position();
        println!("Current FEN: {}", current_board);
        println!("{:?} to move.", game.side_to_move());
        let (next_move_str, move_option) = ask_for_move(input);
        match move_option {
            Some(_) => {
                println!("Move option specified, ending game.");
                break;
            }
            _ => (),
        };

        // convert the SAN to a valid move
        // will repeat the loop if the move is not valid
        let next_move = match ChessMove::from_san(&current_board, &next_move_str) {
            Ok(m) => m,
            Err(e) => {
                println!("Problem with the chess move, try again. Error: {}", e);
                continue;
            }
        };

        game.make_move(next_move);
    }

    game
}

/// Core engine game loop: player and engine alternate turns.
/// The player's input comes through `input`; engine moves come through `rx_out`.
pub(crate) async fn stockfish_game_loop(
    input: &mut dyn BufRead,
    chosen_side: Color,
    tx_in: mpsc::Sender<StockfishInput>,
    mut rx_out: mpsc::Receiver<StockfishOutput>,
) -> Game {
    let mut game = Game::new();

    while game.result().is_none() {
        let current_board = game.current_position();
        println!(
            "Current FEN: {}. {:?} to move.",
            current_board,
            game.side_to_move()
        );

        if chosen_side == game.side_to_move() {
            let (next_move_str, move_option) = ask_for_move(input);
            match move_option {
                Some(_) => {
                    println!("Move option specified, ending game.");
                    break;
                }
                _ => (),
            };

            // convert the SAN to a valid move
            // will repeat the loop if the move is not valid
            let next_move = match ChessMove::from_san(&current_board, &next_move_str) {
                Ok(m) => m,
                Err(e) => {
                    error!("Can't parse move: {e}");
                    println!("Problem with the chess move, try again.");
                    continue;
                }
            };

            // make the move
            game.make_move(next_move);

            // send the move to stockfish
            let next_move_stockfish = StockfishInput::PlayerMove {
                chess_move: next_move,
                fen: game.current_position().to_string(),
            };
            tx_in.send(next_move_stockfish).await.unwrap();
        } else {
            // stockfish's turn
            // wait for a move message from stockfish and play it
            match rx_out.recv().await {
                Some(StockfishOutput::StockfishBestMove { chess_move }) => {
                    debug!(
                        "Received UCI move from stockfish {}{}, making it in our Game copy",
                        chess_move.get_source(),
                        chess_move.get_dest()
                    );
                    game.make_move(chess_move);
                }
                None => {
                    error!("Can't get stockfish output (tx_out dropped), aborting game.");
                    break;
                }
                _ => {}
            };
        }
    }

    game
}

pub(crate) async fn offline_game_stockfish() -> Game {
    use crate::utils::ask_for_side;

    let mut stdin_reader = BufReader::new(std::io::stdin());
    let chosen_side = ask_for_side(&mut stdin_reader);
    debug!("Choosing side {:?}", chosen_side);

    let mut stockfish_child = launch_stockfish("../stockfish16/stockfish-ubuntu-x86-64").await;

    let stockfish_in = stockfish_child
        .stdin
        .take()
        .expect("Stockfish child has no stdin handle");
    let stockfish_out = stockfish_child
        .stdout
        .take()
        .expect("Stockfish child has no stdout handle");

    let (tx_in, rx_in) = mpsc::channel(10);
    let (tx_out, rx_out) = mpsc::channel(40);

    spawn(send_move_to_stockfish(stockfish_in, rx_in));
    spawn(receive_stockfish_best_move(
        tokio::io::BufReader::new(stockfish_out),
        tx_out,
    ));

    let stockfish_config = StockfishInput::Configure {
        level: LEVEL,
        depth: DEPTH,
    };
    tx_in.send(stockfish_config).await.unwrap();

    stockfish_game_loop(&mut stdin_reader, chosen_side, tx_in, rx_out).await
}
