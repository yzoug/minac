extern crate chess;
use chess::{ChessMove, Game, Board};

extern crate lichess_api;
use lichess_api::client::LichessApi;
use lichess_api::error::Result;
use lichess_api::model::Color;
use lichess_api::model::board;
use lichess_api::model::board::stream::events;
use lichess_api::model::board::stream::game;
use lichess_api::model::challenges;

use reqwest::ClientBuilder;
use reqwest::Client;

use tokio::spawn;
use tokio::time::{sleep, Duration};

use futures::stream::StreamExt;

use std::io;
use std::fs;
use std::str::FromStr;

fn get_game_mode() -> u8 {

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

#[tokio::main]
async fn main() -> Result<()> {
    // get the token, panic if you can't read it
    let token = fs::read_to_string("./token.secret")
        .expect("Can't read token.secret file: your Lichess token is needed.");

    // lichess api and http client creation
    let client = ClientBuilder::new().build().unwrap();
    let auth_header = String::from(token).trim().to_string();
    let api = LichessApi::new(client, Some(auth_header.clone()));

    // main program loop
    loop {
        let mode = get_game_mode();
        if mode == 0 {
            offline_game();
        } else if mode == 1 {
            online_game(api.clone()).await?;
        } else {
            println!("Option not supported.");
        }
    }
}

fn offline_game() {
    let mut game = Game::new();

    // while the game is still ongoing
    while game.result().is_none() {
        let current_board = game.current_position();
        println!("Current FEN: {}", current_board);

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

async fn online_game(api: LichessApi<Client>) -> Result<()> {
    // create a new game against a bot
    spawn(setup_bot_game(api.clone()));

    // listen to events and join the game when ready
    stream_events(api).await?;
    Ok(())
}

async fn setup_bot_game(api: LichessApi<Client>) {
    // sleep for a sec, to be sure that the event stream is opened before sending the challenge
    sleep(Duration::from_secs(1)).await;

    // create a game against a bot
    let level = 5;
    let ai_challenge = challenges::AIChallenge {
        level,
        base: challenges::ChallengeBase {
            clock_increment: None,
            clock_limit: None,
            days: None,
            fen: None,
            variant: lichess_api::model::VariantKey::Standard,
        },
        color: lichess_api::model::Color::Random,
    };

    // do the POST request
    match api.challenge_ai(challenges::ai::PostRequest::new(ai_challenge)).await {
        Ok(jsongame) => println!("Game creation OK: {:?}",jsongame),
        Err(e) => println!("Game creation ERROR: {e}"),
    }
}

async fn stream_events(api: LichessApi<Client>) -> Result<()> {
    let stream_request = board::stream::events::GetRequest::new();
    let mut stream = api
        .board_stream_incoming_events(stream_request).await?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(ev) => {
                match ev {
                    events::Event::GameStart { game } => {
                        println!("Game started: {:?}",game);
                        play(&api, game).await?;
                    },
                    events::Event::GameFinish { game } => {
                        println!("Game finished: {:?}",game);
                        break;
                    }
                    _ => println!("Unhandled event type"),
                };
            },
            Err(e) => println!("Error in event loop: {e}"),
        };
    }
    Ok(())
}

async fn play(api: &LichessApi<Client>, lichess_game: events::GameEventInfo) -> Result<()> {
    println!("Playing the game: {:?}", &lichess_game);

    // record the game with the chess crate
    let starting_pos = Board::from_str(&lichess_game.fen)
        .expect("The received starting position from Lichess is invalid");
    let mut game = Game::new_with_board(starting_pos);

    // board state stream
    let request = board::stream::game::GetRequest::new(&lichess_game.game_id);
    let mut stream = api
        .board_stream_board_state(request).await?;

    // if we are white, we first need to input a move before any GameState is received
    match &lichess_game.color {
        Color::White => {
            println!("We are white. Send first fake event to prompt asking for a move.");
            // we fake an event to call our event handling function for the first move
            let fake_first_game_state = game::GameState {
                r#type: None,
                moves: "".to_string(),
                wtime: 0,
                btime: 0,
                winc: 0,
                binc: 0,
                status: "".to_string(),
                winner: None,
                wdraw: None,
                bdraw: None,
                wtakeback: None,
                btakeback: None,
            };
            let fake_first_game_state_event = game::Event::GameState {
                game_state: fake_first_game_state
            };
            handle_current_game_state(
                api,
                &lichess_game.game_id,
                fake_first_game_state_event,
                &mut game,
                &lichess_game.color
            ).await;
        },
        _ => (),
    };

    // stream the state of the board
    while let Some(e) = stream.next().await {
        let mut finished: bool = false;
        // ignore eventual connection errors
        if let Ok(game_state) = e {
            println!("Debug game event: {:?}", game_state);
            finished = handle_current_game_state(
                api,
                &lichess_game.game_id,
                game_state,
                &mut game,
                &lichess_game.color
            ).await;
        }
        if finished {
            println!("Game is finished.");
            break;
        }
    }

    println!("Stream of the played match closed.");
    Ok(())
}

async fn handle_current_game_state(api: &LichessApi<Client>,
    game_id: &str,
    game_state: game::Event,
    game: &mut chess::Game,
    my_color: &Color,
) -> bool {
    let color_in_recorded_game: Color = match game.side_to_move() {
        chess::Color::Black => Color::Black,
        chess::Color::White => Color::White,
    };

    // used to retry an input if wrong move supplied by user
    let mut currently_inputting_move = false;

    match game_state {
        game::Event::OpponentGone { opponent_gone } => {
            println!("Opponent gone: {:?}", opponent_gone);
            true
        },
        game::Event::GameState { game_state } => {
            // if a winner exists, stop here
            if game_state.winner.is_some() {
                return true;
            }
            // loop until valid move supplied
            loop {
                // play an opponent's move in our copy first
                if my_color != &color_in_recorded_game && !currently_inputting_move {
                    // extract the last move, UCI format (ie. source square + dest square, eg. "e2e4")
                    if let Some(last_move) = game_state.moves.split_whitespace().last() {
                        println!("Last move (UCI notation): {}", last_move);

                        // update our recorded game
                        let (source_str_sq, dest_str_sq) = last_move.split_at(2);
                        let source_sq = chess::Square::from_str(source_str_sq).unwrap();
                        let dest_sq = chess::Square::from_str(dest_str_sq).unwrap();
                        // TODO handle promotions
                        let current_move = ChessMove::new(source_sq, dest_sq, None);

                        // try to make the move
                        if !game.make_move(current_move) {
                            // if the f'n returns false, ignore current event: it was our own move
                            println!("The move {:?} is invalid: we probably made this move ourselves.", current_move);
                            return false;
                        } else {
                            println!("Played the opponent's move in our game copy.");
                        }
                    } else {
                        panic!("Should never happen: not my turn, and can't get last move from GameState.");
                    }
                }

                // at this point, we did the move of the opponent in our game copy and we can play
                println!("Current game copy: {:?}.\nYour turn. Enter the SAN move (ex: Nf3):", &game);
                currently_inputting_move = true;

                let mut next_move_str = String::new();
                io::stdin()
                    .read_line(&mut next_move_str)
                    .expect("IO Eroor: failed to read line");

                // convert the SAN to a valid move
                let next_move = match ChessMove::from_san(&game.current_position(), &next_move_str) {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Problem with the chess move, try again. Error: {}", e);
                        continue;
                    },
                };
                // make the move in our game copy
                game.make_move(next_move);

                // send the move to the lichess API
                match api.board_make_move(board::r#move::PostRequest::new(
                    game_id,
                    format!("{}{}", next_move.get_source(), next_move.get_dest()).as_str(),
                    false
                )).await {
                    Ok(_) => (),
                    Err(e) => println!("Error while sending the move to Lichess: {e}"),
                };
                break;

            }

            // return true if an end result has been reached
            println!("Current game status: {}", game.result().is_some());
            game.result().is_some()
        },
        game::Event::GameFull { game_full } => {
            println!("Game full received, handle first move: {:?}", game_full);
            false
        },
        _ => {
            println!("Unhandled game event. Breaking out.");
            true
        },
    }
}
