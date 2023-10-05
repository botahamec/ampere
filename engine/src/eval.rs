use std::{mem::MaybeUninit, num::NonZeroU8, ops::Neg};

use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};

use crate::{transposition_table::TranspositionTableRef, TranspositionTable};

const KING_WORTH: u32 = 2;

fn eval_position(board: CheckersBitBoard) -> f32 {
	let light_pieces = board.pieces_bits() & !board.color_bits();
	let dark_pieces = board.pieces_bits() & board.color_bits();

	let light_peasants = light_pieces & !board.king_bits();
	let dark_peasants = dark_pieces & !board.king_bits();

	let light_kings = light_pieces & board.king_bits();
	let dark_kings = dark_pieces & board.king_bits();

	// if we assume the black player doesn't exist, how good is this for white?
	let light_eval =
		(light_peasants.count_ones() as f32) + ((light_kings.count_ones() * KING_WORTH) as f32);
	let dark_eval =
		(dark_peasants.count_ones() as f32) + ((dark_kings.count_ones() * KING_WORTH) as f32);

	// avoiding a divide by zero error
	if dark_eval + light_eval != 0.0 {
		(dark_eval - light_eval) / (dark_eval + light_eval)
	} else {
		0.0
	}
}

fn eval_jumps(
	mut alpha: f32,
	beta: f32,
	board: CheckersBitBoard,
	table: TranspositionTableRef,
) -> f32 {
	// todo stop checking for jumps twice, but also don't look for slides if there are no jumps
	if PossibleMoves::has_jumps(board) {
		// todo check if this is useful
		// todo make a board for the other player's turn reusable

		let turn = board.turn();
		let mut best_eval = f32::NEG_INFINITY;
		let moves = PossibleMoves::moves(board);

		if moves.is_empty() {
			return -1.0;
		}

		for current_move in moves {
			let board = unsafe { current_move.apply_to(board) };
			let current_eval = if board.turn() != turn {
				-eval_jumps(-beta, -alpha, board, table)
			} else {
				eval_jumps(alpha, beta, board, table)
			};

			table.insert(board, current_eval, unsafe { NonZeroU8::new_unchecked(1) });

			if current_eval >= beta {
				return beta;
			}

			if best_eval < current_eval {
				best_eval = current_eval;
			}
			if alpha < best_eval {
				alpha = best_eval;
			}
		}

		best_eval
	} else if board.turn() == PieceColor::Dark {
		eval_position(board)
	} else {
		-eval_position(board)
	}
}

unsafe fn sort_moves(
	a: &Move,
	b: &Move,
	board: CheckersBitBoard,
	table: TranspositionTableRef,
) -> std::cmp::Ordering {
	let a_entry = table.get_any_depth(a.apply_to(board)).unwrap_or_default();
	let b_entry = table.get_any_depth(b.apply_to(board)).unwrap_or_default();
	a_entry.total_cmp(&b_entry)
}

pub fn negamax(
	depth: u8,
	mut alpha: f32,
	beta: f32,
	board: CheckersBitBoard,
	table: TranspositionTableRef,
) -> f32 {
	if depth < 1 {
		if board.turn() == PieceColor::Dark {
			eval_position(board)
		} else {
			-eval_position(board)
		}
	} else {
		if let Some(entry) = table.get(board, depth) {
			return entry;
		}

		let turn = board.turn();
		let mut best_eval = f32::NEG_INFINITY;
		let mut moves: Vec<Move> = PossibleMoves::moves(board).into_iter().collect();
		moves.sort_unstable_by(|a, b| unsafe { sort_moves(a, b, board, table) });

		if moves.is_empty() {
			return -1.0;
		}

		for current_move in moves {
			let board = unsafe { current_move.apply_to(board) };
			let current_eval = if board.turn() == turn {
				negamax(depth - 1, alpha, beta, board, table)
			} else {
				-negamax(depth - 1, -beta, -alpha, board, table)
			};

			if best_eval < current_eval {
				best_eval = current_eval;
			}

			if alpha < best_eval {
				alpha = best_eval;
			}

			if alpha >= beta {
				return best_eval;
			}
		}

		table.insert(board, best_eval, unsafe { NonZeroU8::new_unchecked(depth) });

		best_eval
	}
}

pub fn current_evaluation(depth: u8, board: CheckersBitBoard, table: TranspositionTableRef) -> f32 {
	let mut alpha = -1.0;
	let mut beta = 1.0;
	for i in 0..depth {
		let mut eval = negamax(i, alpha, beta, board, table);

		if (eval <= alpha) || (eval >= beta) {
			eval = negamax(i, -1.0, 1.0, board, table);
		}

		alpha = f32::max(eval + 0.125, -1.0);
		beta = f32::min(eval + 0.125, 1.0);
	}

	let mut eval = negamax(depth, alpha, beta, board, table);
	if (eval <= alpha) || (eval >= beta) {
		eval = negamax(depth, -1.0, 1.0, board, table);
	}
	eval
}

pub fn best_move(depth: u8, board: CheckersBitBoard, table: TranspositionTableRef) -> Move {
	let moves = PossibleMoves::moves(board).into_iter();
	let mut best_move = None;
	let mut best_eval = std::f32::NEG_INFINITY;
	for current_move in moves {
		let current_board = unsafe { current_move.apply_to(board) };
		let current_eval = if board.turn() == current_board.turn() {
			current_evaluation(depth - 1, current_board, table)
		} else {
			-current_evaluation(depth - 1, current_board, table)
		};

		if current_eval >= best_eval {
			best_eval = current_eval;
			best_move = Some(current_move);
		}
	}

	best_move.unwrap()
}
