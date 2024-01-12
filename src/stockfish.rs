use tokio::io::AsyncWriteExt;
use vampirc_uci::{parse_with_unknown, MessageList};
use tokio::process::{Command, Child, ChildStdin, ChildStdout};
use std::process::Stdio;
use tokio::sync::mpsc;
use tokio::spawn;
use chess::ChessMove;

use crate::online::commands::StockfishMode;

use tokio::io::AsyncReadExt;

const DEPTH: u32 = 5;

pub(crate) async fn launch_stockfish(mode: StockfishMode) -> Child {
    let stockfish = Command::new("../stockfish16/stockfish-ubuntu-x86-64")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Can't spawn Stockfish process."); 

    info!("Stockfish launched.");
    stockfish
}

pub(crate) async fn receive_stockfish_best_move(mut stockfish_out: ChildStdout) {
    // handle current stdout from stockfish to get BestMove
    debug!("ChildStdout recevied: {:?}", stockfish_out);

    let mut buf = Vec::new();
    stockfish_out.read_to_end(&mut buf).await.unwrap();

    debug!("Stockfish says: {:?}", String::from_utf8(buf));
    drop(stockfish_out);
}

pub(crate) async fn send_move_to_stockfish(mut stockfish_in: ChildStdin) {
    debug!("Child recevied: {:?}", stockfish_in);
    stockfish_in.write("uci\nucinewgame\nisready\n".as_bytes()).await.expect("Couldn't write to Stockfish stdin");
    drop(stockfish_in);
}