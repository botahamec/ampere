use std::num::NonZeroU8;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Instant;

use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};

use crate::{
	eval::{eval_position, Evaluation},
	lazysort::LazySort,
	EvaluationTask, Frontend, TranspositionTableRef,
};

unsafe fn sort_moves(
	a: &Move,
	board: CheckersBitBoard,
	table: TranspositionTableRef,
) -> Evaluation {
	table
		.get_any_depth(a.apply_to(board))
		.unwrap_or(Evaluation::DRAW)
}

pub fn negamax(
	depth: u8,
	mut alpha: Evaluation,
	beta: Evaluation,
	board: CheckersBitBoard,
	allowed_moves: Option<Arc<[Move]>>,
	cancel_flag: &AtomicBool,
	task: &EvaluationTask,
) -> (Evaluation, Option<Move>) {
	task.nodes_explored
		.fetch_add(1, std::sync::atomic::Ordering::Release);

	if depth < 1 {
		if board.turn() == PieceColor::Dark {
			(eval_position(board), None)
		} else {
			(-eval_position(board), None)
		}
	} else {
		let table = task.transposition_table;
		if let Some(entry) = table.get(board, depth) {
			return (entry, None);
		}

		let turn = board.turn();
		let mut best_eval = Evaluation::NULL_MIN;
		let mut best_move = None;

		let sort_fn = |m: &Move| unsafe { sort_moves(m, board, table) };
		let sorter: LazySort<Move, _, Evaluation, { PossibleMoves::MAX_POSSIBLE_MOVES }> =
			if let Some(moves) = allowed_moves {
				LazySort::new(moves.iter().cloned(), sort_fn)
			} else {
				let moves = PossibleMoves::moves(board);
				LazySort::new(moves, sort_fn)
			};

		if sorter.is_empty() {
			return (Evaluation::LOSS, None);
		}

		for current_move in sorter.into_iter() {
			if cancel_flag.load(std::sync::atomic::Ordering::Acquire) {
				return (best_eval, best_move);
			}

			let board = unsafe { current_move.apply_to(board) };
			let current_eval = if board.turn() == turn {
				negamax(depth - 1, alpha, beta, board, None, cancel_flag, task)
					.0
					.increment()
			} else {
				-negamax(depth - 1, -beta, -alpha, board, None, cancel_flag, task)
					.0
					.increment()
			};

			if best_eval < current_eval {
				best_eval = current_eval;
				best_move = Some(current_move);
			}

			if alpha < best_eval {
				alpha = best_eval;
			}

			if alpha >= beta {
				return (best_eval, best_move);
			}
		}

		table.insert(board, best_eval, unsafe { NonZeroU8::new_unchecked(depth) });

		(best_eval, best_move)
	}
}

pub fn search(task: Arc<EvaluationTask>, frontend: &dyn Frontend) -> Evaluation {
	let board = task.position;
	let cancel_flag = &task.cancel_flag;

	let allowed_moves = task.allowed_moves.clone();
	let limits = task.limits;
	let max_depth = limits.depth;
	let max_nodes = limits.nodes;
	let max_time = limits.time.map(|d| Instant::now() + d.div_f32(2.0));

	let mut alpha = Evaluation::NULL_MIN;
	let mut beta = Evaluation::NULL_MAX;
	let mut depth = 0;
	let mut eval = Evaluation::DRAW;
	let mut best_move = None;
	loop {
		if let Some(max_depth) = max_depth {
			if depth > max_depth.get() {
				break;
			}
		}

		if let Some(max_time) = max_time {
			if Instant::now() > max_time {
				break;
			}
		}

		if let Some(max_nodes) = max_nodes {
			if task
				.nodes_explored
				.load(std::sync::atomic::Ordering::Acquire)
				> max_nodes.get()
			{
				break;
			}
		}

		let em = negamax(
			depth,
			alpha,
			beta,
			board,
			allowed_moves.clone(),
			cancel_flag,
			&task,
		);

		// prevent incomplete search from overwriting evaluation
		if cancel_flag.load(std::sync::atomic::Ordering::Acquire) {
			break;
		}

		eval = em.0;
		best_move = em.1;

		while (eval <= alpha) || (eval >= beta) {
			let em = negamax(
				depth,
				alpha,
				beta,
				board,
				allowed_moves.clone(),
				cancel_flag,
				&task,
			);

			// prevent incomplete search from overwriting evaluation
			if cancel_flag.load(std::sync::atomic::Ordering::Acquire) {
				break;
			}

			eval = em.0;
			best_move = em.1;

			if eval <= alpha {
				alpha = Evaluation::NULL_MIN;
			} else if eval >= beta {
				beta = Evaluation::NULL_MAX;
			}
		}

		if alpha.is_force_loss() {
			alpha = Evaluation::NULL_MIN;
		} else {
			alpha = eval.add(-0.125);
		}

		if beta.is_force_win() {
			beta = Evaluation::NULL_MAX;
		} else {
			beta = eval.add(0.125);
		}

		depth += 1;
	}

	// ponder
	if let Some(best_move) = best_move {
		// If the best move has not been found yet, then no move will be
		// reported. This should be very rare. This technically is not allowed
		// by the UCI specification, but if someone stops it this quickly, they
		// probably didn't care about the best move anyway.
		frontend.report_best_move(best_move);

		if task.ponder {
			let board = unsafe { best_move.apply_to(board) };

			let mut depth = 0;
			loop {
				if task
					.end_ponder_flag
					.load(std::sync::atomic::Ordering::Acquire)
				{
					break;
				}

				negamax(
					depth,
					Evaluation::NULL_MIN,
					Evaluation::NULL_MAX,
					board,
					None,
					&task.end_ponder_flag,
					&task,
				);

				depth += 1;
			}
		}
	}

	eval
}
