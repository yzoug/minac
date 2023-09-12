extern crate chess;
use chess::{ChessMove, Game};

extern crate lichess_api;
use lichess_api::client::LichessApi;
use lichess_api::model::account;
use lichess_api::model::board;
use lichess_api::error::Result;

use lichess_api::model::board::stream::events::Event;
use tokio::time::{sleep, Duration};
use tokio::spawn;

use futures::stream::StreamExt;

use reqwest::ClientBuilder;

use std::io;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let token = fs::read_to_string("./token.secret")
        .expect("Can't read token.secret file: your Lichess token is needed");

    let result = match get_game_mode() {
        0 => Ok(offline_game()),
        1 => online_game(&token).await,
        _ => Ok({
            println!("Option not supported.");
        })
    };

    match result {
        Ok(_) => Ok(println!("Goodbye!")),
        Err(e) => {
            println!("An error happenned: {:?}", e);
            Err(e)
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


fn get_game_mode() -> u8 {

    println!("minac - minac Is Not A Chessboard
    -----------------------

    All moves must be in SAN (Standard Algebraic Notation).

    Choose either offline (0) or online (1) game:
    > ");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("IO Eroor: failed to read line");

    choice.trim().parse().expect("Please type a number")

}

fn handle_event(event: Event) {
    println!("Event received");
    match event {
        Event::Challenge { challenge } => println!("{:?}",challenge),
        Event::GameStart { game } => println!("{:?}",game),
        Event::GameFinish { game } => println!("{:?}",game),
        _ => (),
    }
}

async fn stream_event_loop(auth_header: String) -> Result<()> {
    let client = ClientBuilder::new().build().unwrap();
    let api = LichessApi::new(client, Some(auth_header));
    println!("Hello from stream_event_loop");
    let stream_request = board::stream::events::GetRequest::new();
    let mut stream = api.board_stream_incoming_events(stream_request).await?;
    println!("Spawned stream");
    while let Some(event) = stream.next().await {
        match event {
            Ok(e) => {
                println!("Event received");
                handle_event(e);
            }
            Err(e) => {
                println!("Error produced: {e}");
            }
        };
    }
    Ok(())
}

async fn online_game(token: &str) -> Result<()> {
    let mut game = Game::new();

    let client = ClientBuilder::new().build().unwrap();

    let auth_header = String::from(token).trim().to_string();
    let api = LichessApi::new(client, Some(auth_header.clone()));

    // seperate runtime for streaming events
    spawn(stream_event_loop(auth_header.clone()));

    // display current profile info
    let profile_request = account::profile::GetRequest::new();
    let profile = api.get_profile(profile_request).await?;
    println!("{:?}", profile);

    // create a game against a bot
    // while the game is still ongoing
    while game.result().is_none() {
        let current_board = game.current_position();

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
                println!("{}", e);
                continue;
            },
        };

        // make the move
        game.make_move(next_move);

    }

    Ok(println!("The game is over. Complete PGN for analysis: {}", game))
}
