use std::fmt::{Debug, Display};
use std::num::NonZeroU8;
use std::ops::Neg;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};

use crate::lazysort::LazySort;
use crate::transposition_table::TranspositionTableRef;
use crate::{EvaluationTask, Frontend};

const KING_WORTH: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Evaluation(i16);

impl Display for Evaluation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_force_win() {
			write!(f, "+M{}", self.force_sequence_length().unwrap())
		} else if self.is_force_loss() {
			write!(f, "-M{}", self.force_sequence_length().unwrap())
		} else {
			write!(f, "{:+}", self.to_f32().unwrap())
		}
	}
}

impl Neg for Evaluation {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl Evaluation {
	const NULL_MAX: Self = Self(i16::MAX);
	const NULL_MIN: Self = Self(i16::MIN + 1);

	pub const WIN: Self = Self(i16::MAX - 1);
	pub const DRAW: Self = Self(0);
	pub const LOSS: Self = Self(i16::MIN + 2);

	// last fourteen bits set to 1
	const FORCE_WIN_THRESHOLD: i16 = 0x3FFF;

	pub fn new(eval: f32) -> Self {
		if eval >= 1.0 {
			return Self::WIN;
		} else if eval <= -1.0 {
			return Self::LOSS;
		}

		Self((eval * 16384.0) as i16)
	}

	pub fn to_f32(self) -> Option<f32> {
		if self.is_force_sequence() {
			return None;
		}

		Some(self.0 as f32 / 16384.0)
	}

	pub fn is_force_win(self) -> bool {
		self.0 > Self::FORCE_WIN_THRESHOLD
	}

	pub fn is_force_loss(self) -> bool {
		self.0 < -Self::FORCE_WIN_THRESHOLD
	}

	pub fn is_force_sequence(self) -> bool {
		self.is_force_win() || self.is_force_loss()
	}

	pub fn force_sequence_length(self) -> Option<u8> {
		if self == Self::NULL_MAX || self == Self::NULL_MIN {
			return None;
		}

		if self.is_force_win() {
			Some((Self::WIN.0 - self.0) as u8)
		} else if self.is_force_loss() {
			Some((self.0 - Self::LOSS.0) as u8)
		} else {
			None
		}
	}

	fn increment(self) -> Self {
		if self.is_force_win() {
			Self(self.0 - 1)
		} else if self.is_force_loss() {
			Self(self.0 + 1)
		} else {
			self
		}
	}

	fn add(self, rhs: f32) -> Self {
		let Some(eval) = self.to_f32() else {
			return self;
		};

		Self::new(eval + rhs)
	}
}

fn eval_position(board: CheckersBitBoard) -> Evaluation {
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
		Evaluation::new((dark_eval - light_eval) / (dark_eval + light_eval))
	} else {
		Evaluation::DRAW
	}
}

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

pub fn evaluate(task: Arc<EvaluationTask>, frontend: &dyn Frontend) -> Evaluation {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn zero_eval() {
		let draw = Evaluation::new(0.0);
		assert_eq!(draw, Evaluation::DRAW);
		assert_eq!(draw.to_f32(), Some(0.0));
		assert_eq!(draw.to_string(), "+0");
	}

	#[test]
	fn comparisons() {
		assert!(Evaluation::NULL_MAX > Evaluation::WIN);
		assert!(Evaluation::WIN > Evaluation::new(0.5));
		assert!(Evaluation::new(0.5) > Evaluation::DRAW);
		assert!(Evaluation::DRAW > Evaluation::new(-0.5));
		assert!(Evaluation::new(-0.5) > Evaluation::LOSS);
		assert!(Evaluation::LOSS > Evaluation::NULL_MIN);
	}

	#[test]
	fn negations() {
		assert_eq!(-Evaluation::NULL_MAX, Evaluation::NULL_MIN);
		assert_eq!(-Evaluation::NULL_MIN, Evaluation::NULL_MAX);
		assert_eq!(-Evaluation::WIN, Evaluation::LOSS);
		assert_eq!(-Evaluation::LOSS, Evaluation::WIN);
		assert_eq!(-Evaluation::DRAW, Evaluation::DRAW);
		assert_eq!(-Evaluation::new(0.5), Evaluation::new(-0.5));
	}
}
