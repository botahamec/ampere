use ai::{CheckersBitBoard, Move, PossibleMoves};
use rayon::prelude::*;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
struct PerftResult {
	result: Vec<(Move, usize)>,
}

pub fn positions(board: CheckersBitBoard, depth: usize) -> usize {
	let moves = PossibleMoves::moves(board);

	if depth == 0 {
		1
	} else {
		let mut total = 0;

		for current_move in moves {
			// safety: we got this move out of the list of possible moves, so it's definitely valid
			let board = unsafe { current_move.apply_to(board) };
			total += positions(board, depth - 1);
		}

		total
	}
}
