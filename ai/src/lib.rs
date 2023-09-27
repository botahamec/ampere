#![feature(new_uninit)]
#![feature(get_mut_unchecked)]

pub use eval::negamax;
pub use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};
pub use transposition_table::{TranspositionTable, TranspositionTableRef};

mod eval;
mod transposition_table;
