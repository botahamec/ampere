#![feature(new_uninit)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]

pub use engine::{
	ActualLimit, Clock, Engine, EvaluationSettings, Frontend, SearchLimit, ENGINE_ABOUT,
	ENGINE_AUTHOR, ENGINE_NAME,
};
pub use eval::Evaluation;
pub use model::{CheckersBitBoard, Move, Piece, PieceColor, PossibleMoves};
pub use transposition_table::{TranspositionTable, TranspositionTableRef};

mod engine;
mod eval;
mod lazysort;
mod search;
mod transposition_table;
