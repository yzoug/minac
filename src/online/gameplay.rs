use crate::online::commands::*;
use crate::online::game_setup::*;
use crate::utils::ask_for_move;

use lichess_api::error::Result;
use lichess_api::client::LichessApi;
use lichess_api::model::Color;
use lichess_api::model::board;
use lichess_api::model::board::stream::game;
use lichess_api::model::board::stream::events;

use chess::{ChessMove, Game, Board};

use reqwest::ClientBuilder;
use reqwest::Client;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use futures::stream::StreamExt;

use std::str::FromStr;
use std::fs;

pub(crate) async fn online_game() -> Result<()> {
    // get the token, panic if you can't read it
    let token = fs::read_to_string("./token.secret")
        .expect("Can't read token.secret file: your Lichess token is needed.");

    // lichess api and http client creation
    let client = ClientBuilder::new().build().unwrap();
    let auth_header = String::from(token).trim().to_string();
    let api = LichessApi::new(client, Some(auth_header));

    // mpsc channel for tasks to send commands
    let (tx, mut rx) = mpsc::channel(10);

    // create a new game against a bot
    spawn(setup_bot_game(tx.clone()));

    // handle received events, send message here when game ready to play
    let stream_events_handle = spawn(stream_events(api.clone(), tx.clone()));
    let mut currently_playing: Option<JoinHandle<Result<()>>> = None;

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::CreateBotGame { bot_game } => {
                api.challenge_ai(bot_game).await?;
            },
            Command::GameStart { game } => {
                if currently_playing.is_some() {
                    currently_playing.unwrap().abort();
                }
                currently_playing = Some(spawn(play(api.clone(), game)));
            },
            Command::GameOver => {
                println!("Game over!");
                if currently_playing.is_some() {
                    currently_playing.unwrap().abort();
                }
                stream_events_handle.abort();
                break;
            }
        }
    }

    Ok(())
}

pub(crate) async fn stream_current_game(
    api: LichessApi<Client>,
    tx: mpsc::Sender<PlayCommand>,
    lichess_game: events::GameEventInfo
) -> Result<()> {

    // stream the state of the board
    let request = board::stream::game::GetRequest::new(&lichess_game.game_id);
    let mut stream = api
        .board_stream_board_state(request).await?;

    // handle the game states
    while let Some(event) = stream.next().await {
        match event {
            Ok(ev) => {
                match ev {
                    game::Event::GameState { game_state } => {
                        // is it my turn?
                        let my_color = &lichess_game.color;
                        // move 1 is white, so n mod 2 gives 1 when it's black's turn, 0 for white's turn
                        let current_color = match game_state.moves.split_whitespace().count() % 2 {
                            1 => {
                                &Color::Black
                            },
                            0 => {
                                &Color::White
                            },
                            _ => {
                                panic!("Math has been broken.");
                            }
                        };

                        // we only handle the game state if it is our turn
                        if current_color == my_color {
                            println!("Handling game state {:?}: my turn", game_state);
                            handle_current_game_state(
                                tx.clone(),
                                Some(game_state),
                            ).await;
                        } else {
                            println!("Ignoring game state {:?}: not my turn", game_state);
                        }
                    },
                    game::Event::OpponentGone { opponent_gone: _ } => {
                        // send msg to play f'n to close all current tasks, and break out of the loop
                        tx.send(PlayCommand::OpponentGone).await.unwrap();
                        break;
                    }
                    _ => println!("Unhandled event type"),
                };
            },
            Err(e) => println!("Error in event loop of current game: {e}"),
        };
    }
    println!("Goodbye from stream_current_game");
    Ok(())

}

pub(crate) async fn play(api: LichessApi<Client>, lichess_game: events::GameEventInfo) -> Result<()> {
    println!("Playing the game: {:?}", &lichess_game);

    // record the game with the chess crate
    let starting_pos = Board::from_str(&lichess_game.fen)
        .expect("The received starting position from Lichess is invalid");
    let mut game = Game::new_with_board(starting_pos);

    // channel to receive play commands from board state
    let (tx, mut rx) = mpsc::channel(10);
    // handle current game stream. Connection will be closed when game is over
    spawn(stream_current_game(api.clone(), tx.clone(), lichess_game.clone()));

    // if we are white, we first need to input a move before any GameState is received by stream_current_game
    match &lichess_game.color {
        Color::White => {
            println!("We are white. Prompt asking for a move.");
            handle_current_game_state(
                tx.clone(),
                None,
            ).await;
        },
        _ => (),
    };

    while let Some(cmd) = rx.recv().await {
        match cmd {
            PlayCommand::MakeMove { chess_move, draw } => {
                let game_chess_move = ChessMove::from_san(
                    &game.current_position(),
                    chess_move.as_str()
                );

                // if the move is valid
                if game_chess_move.is_ok() {
                    let valid_move = game_chess_move.unwrap();

                    // make it in our copy
                    game.make_move(valid_move.clone());

                    let uci_move = format!("{}{}", valid_move.get_source(), valid_move.get_dest());

                    println!("Sending following move to Lichess: {uci_move}");

                    // send it to lichess in UCI format
                    let request = board::r#move::PostRequest::new(
                            &lichess_game.game_id,
                            &uci_move,
                            draw
                    );
                    api.board_make_move(request).await?;
                    println!("Game progression: {}", game);

                } else {
                    println!("The move you entered is not valid. Try again.");
                    handle_current_game_state(
                        tx.clone(),
                        None,
                    ).await;
                }

            },
            PlayCommand::OpponentMove { chess_move } => {
                // only update our game copy
                // move supplied by the API: should be valid, we don't check
                game.make_move(chess_move);
            },
            PlayCommand::Resign => {
                println!("Resigning.");
                let request = board::resign::PostRequest::new(&lichess_game.game_id);
                api.board_resign_game(request).await?;
                break;
            },
            PlayCommand::OpponentGone => {
                println!("Opponent gone.");
                break;
            }
        }
    }

    println!("Stream of the played match closed.");
    Ok(())
}

pub(crate) async fn handle_current_game_state(
    tx: mpsc::Sender<PlayCommand>,
    game_state: Option<game::GameState>,
) {

    match game_state {
        None => {
            println!("No last move supplied: either first move, or wrong move input.");
            let mut draw = false;
            let mut chess_move = ask_for_move();
            if chess_move.ends_with("DRAW") {
                println!("Will ask for draw.");
                draw = true;
                chess_move = chess_move.replace("DRAW", "");
            }
            tx.send(PlayCommand::MakeMove { chess_move, draw }).await.unwrap();
        }
        Some(game_state) => {
            // we only receive game states when it is our turn (i.e. event made by opponent)

            // if a winner exists, stop here
            if game_state.winner.is_some() {
                return;
            }

            // extract the last move from the game state and send it to play f'n
            // UCI format (ie. source square + dest square, eg. "e2e4")
            if let Some(last_move) = game_state.moves.split_whitespace().last() {
                println!("Last move (UCI notation): {}", last_move);
                let (source_str_sq, dest_str_sq) = last_move.split_at(2);
                let source_sq = chess::Square::from_str(source_str_sq).unwrap();
                let dest_sq = chess::Square::from_str(dest_str_sq).unwrap();

                let opponent_move = PlayCommand::OpponentMove {
                    chess_move: ChessMove::new(source_sq, dest_sq, None),
                };
                tx.send(opponent_move).await.unwrap();

                // now that we have the opponent's move, prompt for ours
                let mut move_input = ask_for_move();
                // handle offering draw and resigning
                let mut draw = false;
                if move_input.ends_with("DRAW") {
                    println!("Will ask for draw.");
                    draw = true;
                    move_input = move_input.replace("DRAW", "");
                } else if move_input == "RESIGN" {
                    tx.send(PlayCommand::Resign).await.unwrap();
                }

                let make_move = PlayCommand::MakeMove {
                    chess_move: move_input,
                    draw: draw,
                };
                tx.send(make_move).await.unwrap();

            } else {
                panic!("Should never happen: can't extract move from GameState.\nMoves: {}", game_state.moves);
            }
        },
    }
}