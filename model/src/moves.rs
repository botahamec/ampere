use crate::CheckersBitBoard;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveDirection {
	ForwardLeft = 0,
	ForwardRight = 1,
	BackwardLeft = 2,
	BackwardRight = 3,
}

/// A checkers move
#[derive(Copy, Clone)]
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
