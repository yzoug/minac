extern crate chess;
use chess::{ChessMove, Game};

extern crate lichess_api;
use lichess_api::client::LichessApi;
use lichess_api::error::Result;
use lichess_api::model::board;
use lichess_api::model::board::stream::events::GameEventInfo;
use lichess_api::model::board::stream::events;
use lichess_api::model::challenges;

use reqwest::ClientBuilder;
use reqwest::Client;

use tokio::spawn;
use tokio::time::{sleep, Duration};

use futures::stream::StreamExt;

use std::io;
use std::fs;

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
            Ok(e) => handle_event(&api, e).await?,
            Err(e) => println!("Error in event loop: {e}"),
        };
    }
    Ok(())
}

async fn handle_event(api: &LichessApi<Client>, event: events::Event) -> Result<()> {
    println!("Handling received event...");
    match event {
        events::Event::Challenge { challenge } => println!("Challenge: {:?}",challenge),
        events::Event::GameStart { game } => {
            println!("Game started: {:?}",game);
            play(api, game).await?;
        },
        events::Event::GameFinish { game } => println!("Game finished: {:?}",game),
        _ => println!("Unhandled event type"),
    };
    Ok(())
}

async fn play(api: &LichessApi<Client>, lichess_game: GameEventInfo) -> Result<()> {
    println!("Playing the game: {}", &lichess_game.game_id);

    // record the game with the chess crate
    //let mut game = Game::new();

    let request = board::stream::game::GetRequest::new(&lichess_game.game_id);
    let mut stream = api
        .board_stream_board_state(request).await?;

    // stream the state of the board
    while let Some(e) = stream.next().await {
        // ignore eventual connection errors
        if let Ok(state) = e {
            println!("Debug: what do we receive? {:?}", state);
        }
    }

    println!("Stream closed");
    Ok(())
}