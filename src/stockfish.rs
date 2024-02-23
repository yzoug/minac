use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use vampirc_uci::{parse_with_unknown, Duration, MessageList, UciMessage, UciTimeControl};

use crate::online::commands::{StockfishInput, StockfishOutput};

pub(crate) async fn launch_stockfish(stockfish_bin_path: String) -> Child {
    let stockfish = Command::new(stockfish_bin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Can't spawn Stockfish process.");

    info!("Stockfish launched.");
    stockfish
}

pub(crate) async fn receive_stockfish_best_move(
    stockfish_out: ChildStdout,
    tx: mpsc::Sender<StockfishOutput>,
) {
    // handle current stdout from stockfish to get BestMove
    debug!("ChildStdout recevied: {:?}", stockfish_out);
    let mut reader = BufReader::new(stockfish_out).lines();

    while let Some(next_line) = reader.next_line().await.unwrap() {
        for uci_message in parse_with_unknown(&next_line) {
            trace!("<<<< {:?}", uci_message);
            match uci_message {
                UciMessage::BestMove { best_move, ponder } => {
                    tx.send(StockfishOutput::StockfishBestMove {
                        chess_move: best_move,
                    })
                    .await
                    .unwrap();
                }
                _ => (),
            };
        }
    }
}

pub(crate) async fn send_move_to_stockfish(
    mut stockfish_in: ChildStdin,
    mut rx: mpsc::Receiver<StockfishInput>,
) {
    debug!("ChildStdin recevied: {:?}", stockfish_in);

    // receive stockfish input commands
    while let Some(cmd) = rx.recv().await {
        // the messages vec and string we'll use to send the commands to stockfish
        let mut messages = MessageList::new();
        let mut messages_str = String::new();

        // we give the engine 5 seconds maximum per move
        let time_control = UciTimeControl::TimeLeft {
            white_time: Some(Duration::seconds(5)),
            black_time: Some(Duration::seconds(5)),
            white_increment: None,
            black_increment: None,
            moves_to_go: Some(2),
        };

        match cmd {
            StockfishInput::Configure {
                level,
                depth,
                is_white,
            } => {
                // this is the first expected command, we hence configure a new uci game from the startpos
                messages.push(UciMessage::Uci);
                messages.push(UciMessage::UciNewGame);
                messages.push(UciMessage::Position {
                    startpos: true,
                    fen: None,
                    moves: Vec::new(),
                });

                // we specify the engine strength
                messages.push(UciMessage::SetOption {
                    name: String::from("Skill Level"),
                    value: Some(String::from(level)),
                });
                messages.push(UciMessage::IsReady);

                // if the engine is white, we ask it for a first move
                if is_white {
                    messages.push(UciMessage::Go {
                        time_control: Some(time_control),
                        search_control: None,
                    });
                }
            }
            StockfishInput::PlayerMove { chess_move, fen } => {
                debug!(
                    "Received UCI move from player {}{}, sending it to stockfish",
                    chess_move.get_source(),
                    chess_move.get_dest()
                );
                // send current position with the move we want to play
                let m = UciMessage::Position {
                    startpos: false,
                    fen: Some(vampirc_uci::UciFen(fen)),
                    moves: vec![chess_move],
                };
                messages.push(m);

                messages.push(UciMessage::Go {
                    time_control: Some(time_control),
                    search_control: None,
                });
            }
        }

        // join the messages into a single string seperated by newlines
        for m in messages {
            messages_str.push_str(&m.to_string());
            messages_str.push_str("\n");
        }

        // send the messages
        trace!(">>>> {}", messages_str);
        stockfish_in
            .write(messages_str.as_bytes())
            .await
            .expect("Couldn't write to Stockfish stdin");
        stockfish_in.flush().await.unwrap();
    }
}
