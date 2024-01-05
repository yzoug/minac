use crate::online::commands::*;

use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use futures::stream::StreamExt;
use reqwest::Client;

use lichess_api::model::challenges;
use lichess_api::client::LichessApi;
use lichess_api::model::board::stream::events;
use lichess_api::error::Result;

pub(crate) async fn setup_bot_game(tx: mpsc::Sender<GameCommand>) {
    // sleep for a sec, to be sure that the event stream is opened before sending the challenge
    sleep(Duration::from_secs(1)).await;

    // create a game against a bot
    let level = 1;
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
    let lichess_api_request = GameCommand::CreateBotGame {
        bot_game: challenges::ai::PostRequest::new(ai_challenge),
    };
    match tx.send(lichess_api_request).await {
        Ok(_) => debug!("Setup bot game: message sent successfully to main runtine"),
        Err(e) => error!("Setup bot game: can't send message: {e}"),
    };

}

pub(crate) async fn stream_events(api: LichessApi<Client>, tx: mpsc::Sender<GameCommand>) -> Result<()> {
    let stream_request = events::GetRequest::new();
    let mut stream = api
        .board_stream_incoming_events(stream_request).await?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(ev) => {
                match ev {
                    events::Event::GameStart { game } => {
                        info!("Game started: {:#?}",game);
                        let command = GameCommand::GameStart {
                            game: game,
                        };
                        tx.send(command).await.unwrap();
                    },
                    events::Event::GameFinish { game } => {
                        info!("Game finished: {:#?}",game);
                        tx.send(GameCommand::GameOver).await.unwrap();
                        break;
                    }
                    _ => debug!("Unhandled event type"),
                };
            },
            Err(e) => error!("Error in event loop: {e}"),
        };
    }
    debug!("Goodbye from stream_events");
    Ok(())
}