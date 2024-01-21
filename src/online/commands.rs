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

pub(crate) enum StockfishOutput {
    StockfishEval {
        score: i32,
    },
    StockfishBestMove {
        chess_move: chess::ChessMove,
    },
}

pub(crate) enum StockfishInput {
    PlayerMove {
        chess_move: chess::ChessMove,
        fen: String,
    },
    Configure {
        level: i64,
        depth: i64,
    },
}