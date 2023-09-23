extern crate chess;
extern crate lichess_api;

mod offline;
mod online;
mod utils;

use crate::utils::*;

use lichess_api::error::Result;

#[tokio::main]
async fn main() -> Result<()> {

    // main program loop
    loop {
        let mode = get_game_mode();
        if mode == 0 {
            offline::offline_game();
        } else if mode == 1 {
            online::gameplay::online_game().await?;
        } else {
            println!("Option not supported.");
        }
    }
}