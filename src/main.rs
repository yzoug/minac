extern crate chess;
extern crate lichess_api;
extern crate vampirc_uci;
#[macro_use]
extern crate log;

mod offline;
mod online;
mod stockfish;
mod utils;

use crate::utils::*;

use lichess_api::client::LichessApi;
use lichess_api::error::Result;
use reqwest::ClientBuilder;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("minac v{}", VERSION);

    // main program loop
    loop {
        // get the lichess token, panic if you can't read it
        let token = std::fs::read_to_string("./token.secret")
            .expect("Can't read token.secret file: your Lichess token is needed.");

        // lichess api and http client creation
        let client = ClientBuilder::new()
            .pool_max_idle_per_host(0)
            .build()
            .unwrap();
        let auth_header = String::from(token).trim().to_string();
        let api = LichessApi::new(client, Some(auth_header));

        let mode = get_game_mode();
        if mode == 0 || mode == 1 {
            let game = match mode {
                0 => offline::offline_game_2_players(),
                1 => offline::offline_game_stockfish().await,
                _ => panic!["Math has been broken"],
            };
            println!("The game is over. Complete PGN: {}", game);
            online::gameplay::send_pgn_to_study(api, game.to_string()).await?;
            debug!("Sent the offline game as a chapter in my study");
        } else if mode == 2 {
            online::gameplay::online_game(api).await?;
        } else {
            println!("Option not supported.");
        }
    }
}
