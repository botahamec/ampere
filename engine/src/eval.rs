use std::{
	fmt::{Debug, Display},
	num::NonZeroU8,
	ops::Neg,
};

use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};

use crate::transposition_table::TranspositionTableRef;

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

fn eval_jumps(
	mut alpha: Evaluation,
	beta: Evaluation,
	board: CheckersBitBoard,
	table: TranspositionTableRef,
) -> Evaluation {
	// todo stop checking for jumps twice, but also don't look for slides if there are no jumps
	if PossibleMoves::has_jumps(board) {
		// todo check if this is useful
		// todo make a board for the other player's turn reusable

		let turn = board.turn();
		let mut best_eval = Evaluation::LOSS;
		let moves = PossibleMoves::moves(board);

		if moves.is_empty() {
			return Evaluation::LOSS;
		}

		for current_move in moves {
			let board = unsafe { current_move.apply_to(board) };
			let current_eval = if board.turn() != turn {
				-eval_jumps(-beta, -alpha, board, table).increment()
			} else {
				eval_jumps(alpha, beta, board, table).increment()
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
	let a_entry = table
		.get_any_depth(a.apply_to(board))
		.unwrap_or(Evaluation::DRAW);
	let b_entry = table
		.get_any_depth(b.apply_to(board))
		.unwrap_or(Evaluation::DRAW);
	a_entry.cmp(&b_entry)
}

pub fn negamax(
	depth: u8,
	mut alpha: Evaluation,
	beta: Evaluation,
	board: CheckersBitBoard,
	table: TranspositionTableRef,
) -> Evaluation {
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
		let mut best_eval = Evaluation::LOSS;
		let mut moves: Vec<Move> = PossibleMoves::moves(board).into_iter().collect();

		if moves.is_empty() {
			return Evaluation::LOSS;
		}

		moves.sort_unstable_by(|a, b| unsafe { sort_moves(a, b, board, table) });

		for current_move in moves {
			let board = unsafe { current_move.apply_to(board) };
			let current_eval = if board.turn() == turn {
				negamax(depth - 1, alpha, beta, board, table).increment()
			} else {
				-negamax(depth - 1, -beta, -alpha, board, table).increment()
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

pub fn current_evaluation(
	depth: u8,
	board: CheckersBitBoard,
	table: TranspositionTableRef,
) -> Evaluation {
	let mut alpha = Evaluation::NULL_MIN;
	let mut beta = Evaluation::NULL_MAX;
	for i in 0..depth {
		let mut eval = negamax(i, alpha, beta, board, table);

		while (eval <= alpha) || (eval >= beta) {
			eval = negamax(i, alpha, beta, board, table);

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
	}

	let mut eval = negamax(depth, alpha, beta, board, table);
	if (eval <= alpha) || (eval >= beta) {
		eval = negamax(
			depth,
			Evaluation::NULL_MIN,
			Evaluation::NULL_MAX,
			board,
			table,
		);
	}
	eval
}

pub fn best_move(depth: u8, board: CheckersBitBoard, table: TranspositionTableRef) -> Move {
	let moves = PossibleMoves::moves(board).into_iter();
	let mut best_move = None;
	let mut best_eval = Evaluation::NULL_MIN;
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
