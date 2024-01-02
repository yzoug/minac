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
use tokio::time::{sleep, Duration};
use tokio::task::JoinHandle;
use futures::stream::StreamExt;

use std::str::FromStr;
use std::fs;

pub(crate) async fn online_game() -> Result<()> {
    // get the token, panic if you can't read it
    let token = fs::read_to_string("./token.secret")
        .expect("Can't read token.secret file: your Lichess token is needed.");

    // lichess api and http client creation
    let client = ClientBuilder::new().pool_max_idle_per_host(0).build().unwrap();
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
                println!("<<< Game over! >>>\n");
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
                    game::Event::GameFull { game_full } => {
                        // first event received when opening the stream
                        // if we are black, only sent after white's first move
                        debug!("First event GameFull:\n{:#?}", game_full);
                        if &lichess_game.color == &Color::Black {
                            println!("You are black. Waiting for opponent's move...");
                            // only handle the first game state if we are black
                            handle_current_game_state(
                                tx.clone(),
                                game_full.state,
                            ).await;
                        } else {
                            println!("You are white.");
                        }
                    },
                    game::Event::GameState { game_state } => {
                        // is it my turn?
                        let my_color = &lichess_game.color;
                        // first move is white, so n mod 2 gives 1 when it's black's turn, 0 for white's turn
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
                            debug!("Handling game state {:#?}: my turn", game_state);
                            handle_current_game_state(
                                tx.clone(),
                                Some(game_state),
                            ).await;
                        } else {
                            debug!("Ignoring game state {:#?}: not my turn", game_state);
                        }
                    },
                    game::Event::OpponentGone { opponent_gone: _ } => {
                        // send msg to play f'n to close all current tasks, and break out of the loop
                        tx.send(PlayCommand::OpponentGone).await.unwrap();
                        break;
                    }
                    _ => error!("Unhandled event type"),
                };
            },
            Err(e) => error!("Error in event loop of current game: {e}"),
        };
    }
    debug!("Goodbye from stream_current_game");
    Ok(())

}

pub(crate) async fn play(api: LichessApi<Client>, lichess_game: events::GameEventInfo) -> Result<()> {
    debug!("Playing the game: {:#?}", &lichess_game);

    println!("To offer a draw, enter your next move and DRAW at the end. Example: Qxf5DRAW");
    println!("To resign, enter RESIGN\n");

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
            debug!("We are white. Prompt asking for a move.");
            handle_current_game_state(
                tx.clone(),
                None,
            ).await;
        },
        _ => (),
    };

    while let Some(cmd) = rx.recv().await {
        match cmd {
            PlayCommand::MakeMove { chess_move, option } => {
                let current_position = game.current_position();
                let game_chess_move = ChessMove::from_san(
                    &current_position,
                    chess_move.as_str()
                );
                
                let mut draw = false;
                match option {
                    Some(MoveOption::Resign) => panic!("Should never happen: no make move should be issued if you resign."),
                    Some(MoveOption::Draw) => draw = true,
                    _ => ()
                };

                // if the move is valid
                if game_chess_move.is_ok() {
                    let valid_move = game_chess_move.unwrap();

                    // make it in our copy
                    game.make_move(valid_move.clone());

                    let uci_move = format!("{}{}", valid_move.get_source(), valid_move.get_dest());

                    info!("Sending following move to Lichess: {uci_move}");

                    // send it to lichess in UCI format
                    let request = board::r#move::PostRequest::new(
                            &lichess_game.game_id,
                            &uci_move,
                            draw
                    );
                    api.board_make_move(request).await?;
                    info!("Game progression: {}", game);

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
                println!("Opponent played: {}", chess_move);
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

    info!("Stream of the played match closed.");
    Ok(())
}

pub(crate) async fn handle_current_game_state(
    tx: mpsc::Sender<PlayCommand>,
    game_state: Option<game::GameState>,
) {

    match game_state {
        None => {
            info!("No last move supplied: either first move, or wrong move input.");
            // wait for a bit before grabbing stdin, to let all stdout msg appear
            sleep(Duration::from_millis(100)).await;
            let (chess_move, option) = ask_for_move();
            tx.send(PlayCommand::MakeMove { chess_move, option }).await.unwrap();
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
                let (source_str_sq, dest_str_sq) = last_move.split_at(2);
                let source_sq = chess::Square::from_str(source_str_sq).unwrap();
                let dest_sq = chess::Square::from_str(dest_str_sq).unwrap();

                info!("Last move played (UCI notation): {} to {}", source_str_sq, dest_str_sq);

                let opponent_move = PlayCommand::OpponentMove {
                    chess_move: ChessMove::new(source_sq, dest_sq, None),
                };
                match tx.send(opponent_move).await {
                    Ok(_) => (),
                    Err(e) => panic!("Error while sending opponent move: {e}")
                };

                // now that we have the opponent's move, prompt for ours
                // wait for a bit before grabbing stdin, to let all stdout msg appear
                sleep(Duration::from_millis(100)).await;
                let (chess_move, option) = ask_for_move();
                match option {
                    Some(MoveOption::Resign) => tx.send(PlayCommand::Resign).await.unwrap(),
                    _ => {
                        let make_move = PlayCommand::MakeMove {
                            chess_move,
                            option,
                        };
                        tx.send(make_move).await.unwrap();
                    }
                };
            } else {
                panic!("Should never happen: can't extract move from GameState.\nMoves: {}", game_state.moves);
            }
        },
    }
}
