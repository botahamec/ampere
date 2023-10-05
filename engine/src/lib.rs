#![feature(new_uninit)]
#![feature(get_mut_unchecked)]

pub use eval::{best_move, current_evaluation, negamax};
pub use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};
pub use transposition_table::{TranspositionTable, TranspositionTableRef};

mod eval;
mod tablebase;
mod transposition_table;
