pub use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use std::mem::MaybeUninit;

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
		light_eval / (dark_eval + light_eval)
	} else {
		0.5
	}
}

pub fn eval_singlethreaded(
	depth: usize,
	mut alpha: f32,
	beta: f32,
	board: CheckersBitBoard,
) -> f32 {
	if depth <= 1 {
		eval_position(board)
	} else {
		let turn = board.turn();
		let mut best_eval = f32::NEG_INFINITY;

		for current_move in PossibleMoves::moves(board) {
			let board = unsafe { current_move.apply_to(board) };
			let current_eval = if board.turn() != turn {
				1.0 - eval_singlethreaded(depth - 1, 1.0 - beta, 1.0 - alpha, board)
			} else {
				eval_singlethreaded(depth - 1, alpha, beta, board)
			};

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
	}
}

pub fn eval_multithreaded(depth: usize, alpha: f32, beta: f32, board: CheckersBitBoard) -> f32 {
	if depth <= 1 {
		eval_position(board)
	} else {
		let turn = board.turn();
		let best_eval = Mutex::new(f32::NEG_INFINITY);
		let keep_going = RwLock::new(true);
		let alpha = RwLock::new(alpha);

		let is_still_going = || *keep_going.read();
		let get_alpha = || *alpha.read();
		PossibleMoves::moves(board)
			.into_iter()
			.par_bridge()
			.for_each(|current_move| {
				if is_still_going() {
					let board = unsafe { current_move.apply_to(board) };
					let current_eval = if board.turn() != turn {
						1.0 - eval_singlethreaded(depth - 1, 1.0 - beta, 1.0 - get_alpha(), board)
					} else {
						eval_singlethreaded(depth - 1, get_alpha(), beta, board)
					};

					let mut best = best_eval.lock();
					if current_eval >= beta {
						*best = beta;
						let mut going_val = keep_going.write();
						*going_val = false;
					}

					if *best < current_eval {
						*best = current_eval;
					}
					if get_alpha() < *best {
						let mut alpha = alpha.write();
						*alpha = *best;
					}
				}
			});

		best_eval.into_inner()
	}
}

pub fn best_move(depth: usize, board: CheckersBitBoard) -> Move {
	let mut best_eval = 0.0;
	let mut best_move = MaybeUninit::uninit();
	for current_move in PossibleMoves::moves(board) {
		let current_eval = eval_multithreaded(depth - 1, best_eval, 1.0, unsafe {
			current_move.apply_to(board)
		});
		if current_eval > best_eval {
			best_eval = current_eval;
			best_move = MaybeUninit::new(current_move);
		}
	}

	unsafe { best_move.assume_init() }
}
