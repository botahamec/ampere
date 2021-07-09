use crate::CheckersBitBoard;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveDirection {
	ForwardLeft = 0,
	ForwardRight = 1,
	BackwardLeft = 2,
	BackwardRight = 3,
}

#[derive(Copy, Clone)]
pub struct Move {
	start: u32,
	direction: MoveDirection,
	jump: bool,
}

impl Move {
	pub const fn new(start: usize, direction: MoveDirection, jump: bool) -> Self {
		Self {
			start: start as u32,
			direction,
			jump,
		}
	}

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
