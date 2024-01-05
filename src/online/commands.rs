use lichess_api::model::challenges;
use lichess_api::model::board::stream::events::GameEventInfo;

pub(crate) enum GameCommand {
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
        option: Option<MoveOption>,
    },
    OpponentMove {
        chess_move: chess::ChessMove,
    },
    Resign,
    OpponentGone,
}

pub(crate) enum MoveOption {
    Draw,
    Resign,
}