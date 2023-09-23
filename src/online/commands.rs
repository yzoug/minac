use lichess_api::model::challenges;
use lichess_api::model::board::stream::events::GameEventInfo;

pub(crate) enum Command {
    CreateBotGame {
        bot_game: challenges::ai::PostRequest,
    },
    GameStart {
        game: GameEventInfo,
    },
    GameOver,
}

pub(crate) enum PlayCommand {
    MakeMove {
        chess_move: String,
        draw: bool,
    },
    OpponentMove {
        chess_move: chess::ChessMove,
    },
    Resign,
    OpponentGone,
}