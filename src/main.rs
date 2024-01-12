extern crate chess;
extern crate lichess_api;
#[macro_use]
extern crate log;

mod offline;
mod online;
mod utils;
mod stockfish;

use crate::utils::*;

use lichess_api::error::Result;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {

    env_logger::init();

    info!("minac v{}", VERSION);

    // main program loop
    loop {
        let mode = get_game_mode();
        if mode == 0 {
            offline::offline_game_2_players();
        } else if mode == 1 {
            offline::offline_game_stockfish().await;
        } else if mode == 2 {
            online::gameplay::online_game().await?;
        } else {
            println!("Option not supported.");
        }
    }
}