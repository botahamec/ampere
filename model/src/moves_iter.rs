use crate::moves::{Move, MoveDirection};
use crate::possible_moves::PossibleMoves;

const FORWARD_LEFT_SLIDE_SQUARES: [usize; 33] = [
	1, 3, 3, 4, 6, 6, 7, 8, 9, 12, 12, 12, 13, 14, 15, 16, 17, 19, 19, 20, 21, 22, 23, 24, 27, 27,
	27, 28, 29, 30, 32, 32, 32,
];
const FORWARD_RIGHT_SLIDE_SQUARES: [usize; 33] = [
	2, 2, 3, 4, 6, 6, 7, 8, 10, 10, 12, 12, 13, 14, 15, 16, 18, 18, 19, 20, 21, 22, 23, 24, 26, 26,
	27, 28, 29, 30, 32, 32, 32,
];
const BACKWARD_LEFT_SLIDE_SQUARES: [usize; 33] = [
	1, 3, 3, 4, 5, 7, 7, 8, 9, 11, 11, 13, 13, 14, 15, 16, 17, 19, 19, 20, 21, 22, 23, 24, 25, 27,
	27, 28, 29, 30, 31, 32, 32,
];
const BACKWARD_RIGHT_SLIDE_SQUARES: [usize; 33] = [
	2, 2, 3, 4, 5, 7, 7, 8, 10, 10, 11, 13, 13, 14, 15, 16, 19, 19, 19, 20, 21, 22, 23, 24, 26, 26,
	27, 28, 29, 30, 31, 32, 32,
];

const FORWARD_LEFT_JUMP_SQUARES: [usize; 33] = [
	1, 6, 6, 6, 6, 6, 7, 8, 9, 12, 12, 12, 13, 14, 15, 16, 17, 20, 20, 20, 21, 22, 23, 28, 28, 28,
	28, 28, 29, 32, 32, 32, 32,
];
const FORWARD_RIGHT_JUMP_SQUARES: [usize; 33] = [
	2, 2, 3, 6, 6, 6, 7, 12, 12, 12, 12, 12, 13, 14, 15, 18, 18, 18, 19, 20, 21, 22, 23, 26, 26,
	26, 27, 28, 29, 32, 32, 32, 32,
];
const BACKWARD_LEFT_JUMP_SQUARES: [usize; 33] = [
	4, 4, 4, 4, 5, 8, 8, 8, 9, 14, 14, 14, 14, 14, 15, 16, 17, 20, 20, 20, 21, 22, 23, 24, 25, 28,
	28, 28, 29, 30, 31, 32, 32,
];
const BACKWARD_RIGHT_JUMP_SQUARES: [usize; 33] = [
	2, 2, 3, 4, 5, 10, 10, 10, 10, 10, 11, 14, 14, 14, 15, 20, 20, 20, 20, 20, 21, 22, 23, 26, 26,
	26, 27, 28, 29, 30, 31, 32, 32,
];

static SLIDE_ARRAYS: [[usize; 33]; 4] = [
	FORWARD_LEFT_SLIDE_SQUARES,
	FORWARD_RIGHT_SLIDE_SQUARES,
	BACKWARD_LEFT_SLIDE_SQUARES,
	BACKWARD_RIGHT_SLIDE_SQUARES,
];

static JUMP_ARRAYS: [[usize; 33]; 4] = [
	FORWARD_LEFT_JUMP_SQUARES,
	FORWARD_RIGHT_JUMP_SQUARES,
	BACKWARD_LEFT_JUMP_SQUARES,
	BACKWARD_RIGHT_JUMP_SQUARES,
];

pub struct PossibleMovesIter {
	possible_moves: PossibleMoves,
	current_square: usize,
	current_direction: MoveDirection,
	movers: u32,
	squares: &'static [usize; 33],
}

impl From<PossibleMoves> for PossibleMovesIter {
	fn from(possible_moves: PossibleMoves) -> Self {
		Self {
			possible_moves,
			current_square: 0,
			current_direction: MoveDirection::ForwardLeft,
			movers: possible_moves.forward_left_bits(),
			squares: unsafe {
				if possible_moves.can_jump() {
					JUMP_ARRAYS.get_unchecked(0)
				} else {
					SLIDE_ARRAYS.get_unchecked(0)
				}
			},
		}
	}
}

impl Iterator for PossibleMovesIter {
	type Item = Move;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if self.current_square == 32 {
				if self.current_direction != MoveDirection::BackwardRight {
					self.current_square = 0;
					// safety: only results in undefined variant if equal to backward right
					// this has already been checked for
					self.current_direction =
						unsafe { std::mem::transmute((self.current_direction as u8) + 1) };
					self.movers = self
						.possible_moves
						.get_direction_bits(self.current_direction);

					// safety: the max value of the enum is 3
					unsafe {
						self.squares = &*(self.squares as *const [usize; 33]).add(1);
					}
				} else {
					return None;
				}
			}

			if (self.movers >> self.current_square) & 1 != 0 {
				let next_move = Move::new(
					self.current_square,
					self.current_direction,
					self.possible_moves.can_jump(),
				);

				// safety: self.current_square will never be > 32
				// squares does not contain such a value
				unsafe {
					self.current_square = *self.squares.get_unchecked(self.current_square);
				}

				return Some(next_move);
			}

			if self.current_square != 32 {
				// safety: self.current_square will never be > 32
				// squares does not contain such a value
				unsafe {
					self.current_square = *self.squares.get_unchecked(self.current_square);
				}
			}
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(
			0,
			Some(
				(32 - self.current_square)
					+ 32 * match self.current_direction {
						MoveDirection::ForwardLeft => 3,
						MoveDirection::ForwardRight => 2,
						MoveDirection::BackwardLeft => 1,
						MoveDirection::BackwardRight => 0,
					},
			),
		)
	}
}
