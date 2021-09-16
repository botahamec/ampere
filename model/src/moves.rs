use crate::{CheckersBitBoard, SquareCoordinate};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MoveDirection {
	ForwardLeft = 0,
	ForwardRight = 1,
	BackwardLeft = 2,
	BackwardRight = 3,
}

/// A checkers move
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Move {
	/// The position of the piece to move
	start: u32,

	/// The direction to move to
	direction: MoveDirection,

	/// Whether or not it's a jump
	jump: bool,
}

impl Move {
	/// Create a new move
	///
	/// # Arguments
	///
	/// * `start` - The location of the piece that should move
	/// * `direction` - The direction the piece should move in
	/// * `jump` - Whether or not the piece should jump
	pub const fn new(start: usize, direction: MoveDirection, jump: bool) -> Self {
		Self {
			start: start as u32,
			direction,
			jump,
		}
	}

	/// The stating position of the move
	pub const fn start(self) -> u32 {
		self.start
	}

	/// The direction the move goes in
	pub const fn direction(self) -> MoveDirection {
		self.direction
	}

	/// Returns `true` if the move is a jump
	pub const fn is_jump(self) -> bool {
		self.jump
	}

	/// Calculates the value of the end position of the move
	pub const fn end_position(self) -> usize {
		let dest = match self.jump {
			false => match self.direction {
				MoveDirection::ForwardLeft => (self.start + 7) % 32,
				MoveDirection::ForwardRight => (self.start + 1) % 32,
				MoveDirection::BackwardLeft => self.start.wrapping_sub(1) % 32,
				MoveDirection::BackwardRight => self.start.wrapping_sub(7) % 32,
			},
			true => match self.direction {
				MoveDirection::ForwardLeft => (self.start + 14) % 32,
				MoveDirection::ForwardRight => (self.start + 2) % 32,
				MoveDirection::BackwardLeft => self.start.wrapping_sub(2) % 32,
				MoveDirection::BackwardRight => self.start.wrapping_sub(14) % 32,
			},
		};
		dest as usize
	}

	/// Apply the move to a board. This does not mutate the original board,
	/// but instead returns a new one.
	///
	/// # Arguments
	///
	/// * `board` - The board to apply the move to
	///
	/// # Panics
	///
	/// Panics if the starting position of this move is greater than or equal to 32
	///
	/// # Safety
	///
	/// Applying an illegal move to the board is undefined behavior.
	/// This functions results in undefined behavior if:
	/// * The piece moves in a direction which would move it outside of the board
	/// * The starting position of this move doesn't contain a piece
	pub const unsafe fn apply_to(self, board: CheckersBitBoard) -> CheckersBitBoard {
		match self.jump {
			false => match self.direction {
				MoveDirection::ForwardLeft => {
					board.move_piece_forward_left_unchecked(self.start as usize)
				}
				MoveDirection::ForwardRight => {
					board.move_piece_forward_right_unchecked(self.start as usize)
				}
				MoveDirection::BackwardLeft => {
					board.move_piece_backward_left_unchecked(self.start as usize)
				}
				MoveDirection::BackwardRight => {
					board.move_piece_backward_right_unchecked(self.start as usize)
				}
			},
			true => match self.direction {
				MoveDirection::ForwardLeft => {
					board.jump_piece_forward_left_unchecked(self.start as usize)
				}
				MoveDirection::ForwardRight => {
					board.jump_piece_forward_right_unchecked(self.start as usize)
				}
				MoveDirection::BackwardLeft => {
					board.jump_piece_backward_left_unchecked(self.start as usize)
				}
				MoveDirection::BackwardRight => {
					board.jump_piece_backward_right_unchecked(self.start as usize)
				}
			},
		}
	}
}

impl Display for Move {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}-{}",
			SquareCoordinate::from_value(self.start as usize),
			SquareCoordinate::from_value(self.end_position())
		)
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use proptest::prelude::*;

	proptest! {
		#[test]
		fn new(start in 0usize..32, jump in proptest::bool::ANY) {
			let direction = MoveDirection::ForwardLeft;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.start as usize, start);
			assert_eq!(move_test.direction, direction);
			assert_eq!(move_test.jump, jump);

			let direction = MoveDirection::ForwardRight;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.start as usize, start);
			assert_eq!(move_test.direction, direction);
			assert_eq!(move_test.jump, jump);

			let direction = MoveDirection::BackwardLeft;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.start as usize, start);
			assert_eq!(move_test.direction, direction);
			assert_eq!(move_test.jump, jump);

			let direction = MoveDirection::BackwardRight;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.start as usize, start);
			assert_eq!(move_test.direction, direction);
			assert_eq!(move_test.jump, jump);
		}

		#[test]
		fn start(start in 0usize..32, jump in proptest::bool::ANY) {
			let direction = MoveDirection::ForwardLeft;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.start(), start as u32);
		}

		#[test]
		fn direction(start in 0usize..32, jump in proptest::bool::ANY) {
			let direction = MoveDirection::ForwardLeft;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.direction(), direction);

			let direction = MoveDirection::ForwardRight;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.direction(), direction);

			let direction = MoveDirection::BackwardLeft;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.direction(), direction);

			let direction = MoveDirection::BackwardRight;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.direction(), direction);
		}

		#[test]
		fn is_jump(start in 0usize..32, jump in proptest::bool::ANY) {
			let direction = MoveDirection::ForwardLeft;
			let move_test = Move::new(start, direction, jump);
			assert_eq!(move_test.is_jump(), jump);
		}
	}

	#[test]
	fn end_position_forward_left_slide() {
		let direction = MoveDirection::ForwardLeft;
		let start = 8;
		let move_test = Move::new(start, direction, false);
		assert_eq!(move_test.end_position(), 15);
	}

	#[test]
	fn end_position_forward_right_slide() {
		let direction = MoveDirection::ForwardRight;
		let start = 26;
		let move_test = Move::new(start, direction, false);
		assert_eq!(move_test.end_position(), 27);
	}

	#[test]
	fn end_position_backward_right_slide() {
		let direction = MoveDirection::BackwardRight;
		let start = 2;
		let move_test = Move::new(start, direction, false);
		assert_eq!(move_test.end_position(), 27);
	}

	#[test]
	fn end_position_backward_left_slide() {
		let direction = MoveDirection::BackwardLeft;
		let start = 16;
		let move_test = Move::new(start, direction, false);
		assert_eq!(move_test.end_position(), 15);
	}

	#[test]
	fn end_position_forward_left_jump() {
		let direction = MoveDirection::ForwardLeft;
		let start = 8;
		let move_test = Move::new(start, direction, true);
		assert_eq!(move_test.end_position(), 22);
	}

	#[test]
	fn end_position_forward_right_jump() {
		let direction = MoveDirection::ForwardRight;
		let start = 26;
		let move_test = Move::new(start, direction, true);
		assert_eq!(move_test.end_position(), 28);
	}

	#[test]
	fn end_position_backward_right_jump() {
		let direction = MoveDirection::BackwardRight;
		let start = 2;
		let move_test = Move::new(start, direction, true);
		assert_eq!(move_test.end_position(), 20);
	}

	#[test]
	fn end_position_backward_left_jump() {
		let direction = MoveDirection::BackwardLeft;
		let start = 16;
		let move_test = Move::new(start, direction, true);
		assert_eq!(move_test.end_position(), 14);
	}
}
