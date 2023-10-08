use std::{
	cmp::Ordering,
	fmt::{Debug, Display},
	num::NonZeroU8,
	ops::Neg,
};

use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};

use crate::transposition_table::TranspositionTableRef;

const KING_WORTH: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evaluation {
	ForceWin(u32),
	FloatEval(f32),
	ForceLoss(u32),
}

impl PartialOrd for Evaluation {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for Evaluation {}

impl Ord for Evaluation {
	fn cmp(&self, other: &Self) -> Ordering {
		match self {
			Evaluation::ForceWin(moves) => match other {
				Self::ForceWin(other_moves) => moves.cmp(other_moves).reverse(),
				_ => Ordering::Greater,
			},
			Evaluation::FloatEval(eval) => match other {
				Self::ForceWin(_) => Ordering::Less,
				Self::FloatEval(other_eval) => eval.total_cmp(other_eval),
				Self::ForceLoss(_) => Ordering::Greater,
			},
			Evaluation::ForceLoss(moves) => match other {
				Self::ForceLoss(other_moves) => moves.cmp(other_moves),
				_ => Ordering::Less,
			},
		}
	}
}

impl Display for Evaluation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ForceWin(moves) => write!(f, "+M{moves}"),
			Self::FloatEval(eval) => write!(f, "{eval:+}"),
			Self::ForceLoss(moves) => write!(f, "-M{moves}"),
		}
	}
}

impl Neg for Evaluation {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Self::ForceWin(moves) => Self::ForceLoss(moves),
			Self::FloatEval(eval) => Self::FloatEval(-eval),
			Self::ForceLoss(moves) => Self::ForceWin(moves),
		}
	}
}

impl Evaluation {
	const WIN: Self = Self::ForceWin(0);
	const LOSS: Self = Self::ForceLoss(0);
	const DRAW: Self = Self::FloatEval(0.0);

	fn increment(self) -> Self {
		match self {
			Self::ForceWin(moves) => Self::ForceWin(moves + 1),
			Self::FloatEval(_) => self,
			Self::ForceLoss(moves) => Self::ForceLoss(moves + 1),
		}
	}

	fn add(self, rhs: f32) -> Self {
		if let Self::FloatEval(eval) = self {
			let eval = eval + rhs;
			if eval >= 1.0 {
				Self::WIN
			} else if eval <= 0.0 {
				Self::LOSS
			} else {
				Self::FloatEval(eval)
			}
		} else {
			self
		}
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
		Evaluation::FloatEval((dark_eval - light_eval) / (dark_eval + light_eval))
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
	let mut alpha = Evaluation::LOSS;
	let mut beta = Evaluation::WIN;
	for i in 0..depth {
		let mut eval = negamax(i, alpha, beta, board, table);

		while (eval < alpha) || (eval > beta) {
			eval = negamax(i, alpha, beta, board, table);

			if eval <= alpha {
				alpha = Evaluation::LOSS;
			} else if eval >= beta {
				beta = Evaluation::WIN;
			}
		}

		alpha = Evaluation::max(eval.add(0.125), Evaluation::LOSS);
		beta = Evaluation::min(eval.add(-0.125), Evaluation::WIN);
	}

	let mut eval = negamax(depth, alpha, beta, board, table);
	if (eval <= alpha) || (eval >= beta) {
		eval = negamax(depth, Evaluation::LOSS, Evaluation::WIN, board, table);
	}
	eval
}

pub fn best_move(depth: u8, board: CheckersBitBoard, table: TranspositionTableRef) -> Move {
	let moves = PossibleMoves::moves(board).into_iter();
	let mut best_move = None;
	let mut best_eval = Evaluation::LOSS;
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
