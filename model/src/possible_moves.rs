use crate::moves::{Move, MoveDirection};
use crate::{CheckersBitBoard, PieceColor};

use std::mem::MaybeUninit;

// The maximum number of available moves in any given position
pub const POSSIBLE_MOVES_ITER_SIZE: usize = 50;

/// A struct containing the possible moves in a particular checkers position
#[derive(Copy, Clone, Debug)]
pub struct PossibleMoves {
	forward_left_movers: u32,
	forward_right_movers: u32,
	backward_left_movers: u32,
	backward_right_movers: u32,
}

/// An iterator of possible checkers moves for a particular position
pub struct PossibleMovesIter {
	/// A pointer to an array of possibly uninitialized checkers moves
	moves: [MaybeUninit<Move>; POSSIBLE_MOVES_ITER_SIZE],

	/// The current index into the moves array
	index: usize,

	// The number of initialized moves in the array
	length: usize,
}

impl PossibleMovesIter {
	fn add_slide_forward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_left_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardLeft, false));
			self.length += 1;
		}
	}

	fn add_slide_forward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_right_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardRight, false));
			self.length += 1;
		}
	}

	fn add_slide_backward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_left_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardLeft, false));
			self.length += 1;
		}
	}

	fn add_slide_backward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_right_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardRight, false));
			self.length += 1;
		}
	}

	fn add_jump_forward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_left_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardLeft, true));
			self.length += 1;
		}
	}

	fn add_jump_forward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_right_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardRight, true));
			self.length += 1;
		}
	}

	fn add_jump_backward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_left_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardLeft, true));
			self.length += 1;
		}
	}

	fn add_jump_backward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_right_movers >> SQUARE) & 1 != 0 {
			debug_assert!(self.length < POSSIBLE_MOVES_ITER_SIZE);
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardRight, true));
			self.length += 1;
		}
	}
}

unsafe impl Send for PossibleMovesIter {}

impl Iterator for PossibleMovesIter {
	type Item = Move;

	fn next(&mut self) -> Option<Self::Item> {
		if self.length > self.index {
			debug_assert!(self.index < POSSIBLE_MOVES_ITER_SIZE);
			let next_move = unsafe { self.moves.as_ref().get_unchecked(self.index).assume_init() };
			self.index += 1;
			Some(next_move)
		} else {
			None
		}
	}

	// TODO test
	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.length - self.index;
		(remaining, Some(remaining))
	}

	// TODO test
	fn count(self) -> usize
	where
		Self: Sized,
	{
		self.length - self.index
	}

	// TODO test
	fn last(self) -> Option<Self::Item>
	where
		Self: Sized,
	{
		debug_assert!(self.length <= POSSIBLE_MOVES_ITER_SIZE);
		if self.length == 0 {
			None
		} else {
			Some(unsafe {
				self.moves
					.as_ref()
					.get_unchecked(self.length - 1)
					.assume_init()
			})
		}
	}

	// TODO test
	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		if self.length == 0 || self.length - self.index < n {
			None
		} else {
			self.index += n;
			let current_move =
				unsafe { self.moves.as_ref().get_unchecked(self.index).assume_init() };
			self.index += 1;
			Some(current_move)
		}
	}
}

impl IntoIterator for PossibleMoves {
	type Item = Move;
	type IntoIter = PossibleMovesIter;

	// TODO test
	fn into_iter(self) -> Self::IntoIter {
		let moves = [MaybeUninit::uninit(); POSSIBLE_MOVES_ITER_SIZE];
		let mut iter = PossibleMovesIter {
			moves,
			index: 0,
			length: 0,
		};

		if self.can_jump() {
			iter.add_jump_forward_left::<0>(self);
			iter.add_jump_forward_left::<1>(self);
			iter.add_jump_forward_left::<6>(self);
			iter.add_jump_forward_left::<7>(self);
			iter.add_jump_forward_left::<8>(self);
			iter.add_jump_forward_left::<9>(self);
			iter.add_jump_forward_left::<12>(self);
			iter.add_jump_forward_left::<13>(self);
			iter.add_jump_forward_left::<14>(self);
			iter.add_jump_forward_left::<15>(self);
			iter.add_jump_forward_left::<16>(self);
			iter.add_jump_forward_left::<17>(self);
			iter.add_jump_forward_left::<20>(self);
			iter.add_jump_forward_left::<21>(self);
			iter.add_jump_forward_left::<22>(self);
			iter.add_jump_forward_left::<23>(self);
			iter.add_jump_forward_left::<28>(self);
			iter.add_jump_forward_left::<29>(self);

			iter.add_jump_forward_right::<2>(self);
			iter.add_jump_forward_right::<3>(self);
			iter.add_jump_forward_right::<6>(self);
			iter.add_jump_forward_right::<7>(self);
			iter.add_jump_forward_right::<12>(self);
			iter.add_jump_forward_right::<13>(self);
			iter.add_jump_forward_right::<14>(self);
			iter.add_jump_forward_right::<15>(self);
			iter.add_jump_forward_right::<18>(self);
			iter.add_jump_forward_right::<19>(self);
			iter.add_jump_forward_right::<20>(self);
			iter.add_jump_forward_right::<21>(self);
			iter.add_jump_forward_right::<22>(self);
			iter.add_jump_forward_right::<23>(self);
			iter.add_jump_forward_right::<26>(self);
			iter.add_jump_forward_right::<27>(self);
			iter.add_jump_forward_right::<28>(self);
			iter.add_jump_forward_right::<29>(self);

			iter.add_jump_backward_left::<4>(self);
			iter.add_jump_backward_left::<5>(self);
			iter.add_jump_backward_left::<8>(self);
			iter.add_jump_backward_left::<9>(self);
			iter.add_jump_backward_left::<14>(self);
			iter.add_jump_backward_left::<15>(self);
			iter.add_jump_backward_left::<16>(self);
			iter.add_jump_backward_left::<17>(self);
			iter.add_jump_backward_left::<20>(self);
			iter.add_jump_backward_left::<21>(self);
			iter.add_jump_backward_left::<22>(self);
			iter.add_jump_backward_left::<23>(self);
			iter.add_jump_backward_left::<24>(self);
			iter.add_jump_backward_left::<25>(self);
			iter.add_jump_backward_left::<28>(self);
			iter.add_jump_backward_left::<29>(self);
			iter.add_jump_backward_left::<30>(self);
			iter.add_jump_backward_left::<31>(self);

			iter.add_jump_backward_right::<2>(self);
			iter.add_jump_backward_right::<3>(self);
			iter.add_jump_backward_right::<4>(self);
			iter.add_jump_backward_right::<5>(self);
			iter.add_jump_backward_right::<10>(self);
			iter.add_jump_backward_right::<11>(self);
			iter.add_jump_backward_right::<14>(self);
			iter.add_jump_backward_right::<15>(self);
			iter.add_jump_backward_right::<20>(self);
			iter.add_jump_backward_right::<21>(self);
			iter.add_jump_backward_right::<22>(self);
			iter.add_jump_backward_right::<23>(self);
			iter.add_jump_backward_right::<26>(self);
			iter.add_jump_backward_right::<27>(self);
			iter.add_jump_backward_right::<28>(self);
			iter.add_jump_backward_right::<29>(self);
			iter.add_jump_backward_right::<30>(self);
			iter.add_jump_backward_right::<31>(self);
		} else {
			iter.add_slide_forward_left::<0>(self);
			iter.add_slide_forward_left::<1>(self);
			iter.add_slide_forward_left::<3>(self);
			iter.add_slide_forward_left::<4>(self);
			iter.add_slide_forward_left::<6>(self);
			iter.add_slide_forward_left::<7>(self);
			iter.add_slide_forward_left::<8>(self);
			iter.add_slide_forward_left::<9>(self);
			iter.add_slide_forward_left::<12>(self);
			iter.add_slide_forward_left::<13>(self);
			iter.add_slide_forward_left::<14>(self);
			iter.add_slide_forward_left::<15>(self);
			iter.add_slide_forward_left::<16>(self);
			iter.add_slide_forward_left::<17>(self);
			iter.add_slide_forward_left::<19>(self);
			iter.add_slide_forward_left::<20>(self);
			iter.add_slide_forward_left::<21>(self);
			iter.add_slide_forward_left::<22>(self);
			iter.add_slide_forward_left::<23>(self);
			iter.add_slide_forward_left::<24>(self);
			iter.add_slide_forward_left::<27>(self);
			iter.add_slide_forward_left::<28>(self);
			iter.add_slide_forward_left::<29>(self);
			iter.add_slide_forward_left::<30>(self);

			iter.add_slide_forward_right::<0>(self);
			iter.add_slide_forward_right::<2>(self);
			iter.add_slide_forward_right::<3>(self);
			iter.add_slide_forward_right::<4>(self);
			iter.add_slide_forward_right::<6>(self);
			iter.add_slide_forward_right::<7>(self);
			iter.add_slide_forward_right::<8>(self);
			iter.add_slide_forward_right::<10>(self);
			iter.add_slide_forward_right::<12>(self);
			iter.add_slide_forward_right::<13>(self);
			iter.add_slide_forward_right::<14>(self);
			iter.add_slide_forward_right::<15>(self);
			iter.add_slide_forward_right::<16>(self);
			iter.add_slide_forward_right::<18>(self);
			iter.add_slide_forward_right::<19>(self);
			iter.add_slide_forward_right::<20>(self);
			iter.add_slide_forward_right::<21>(self);
			iter.add_slide_forward_right::<22>(self);
			iter.add_slide_forward_right::<23>(self);
			iter.add_slide_forward_right::<24>(self);
			iter.add_slide_forward_right::<26>(self);
			iter.add_slide_forward_right::<27>(self);
			iter.add_slide_forward_right::<28>(self);
			iter.add_slide_forward_right::<29>(self);
			iter.add_slide_forward_right::<30>(self);

			iter.add_slide_backward_left::<1>(self);
			iter.add_slide_backward_left::<3>(self);
			iter.add_slide_backward_left::<4>(self);
			iter.add_slide_backward_left::<5>(self);
			iter.add_slide_backward_left::<7>(self);
			iter.add_slide_backward_left::<8>(self);
			iter.add_slide_backward_left::<9>(self);
			iter.add_slide_backward_left::<11>(self);
			iter.add_slide_backward_left::<13>(self);
			iter.add_slide_backward_left::<14>(self);
			iter.add_slide_backward_left::<15>(self);
			iter.add_slide_backward_left::<16>(self);
			iter.add_slide_backward_left::<17>(self);
			iter.add_slide_backward_left::<19>(self);
			iter.add_slide_backward_left::<20>(self);
			iter.add_slide_backward_left::<21>(self);
			iter.add_slide_backward_left::<22>(self);
			iter.add_slide_backward_left::<23>(self);
			iter.add_slide_backward_left::<24>(self);
			iter.add_slide_backward_left::<25>(self);
			iter.add_slide_backward_left::<27>(self);
			iter.add_slide_backward_left::<28>(self);
			iter.add_slide_backward_left::<29>(self);
			iter.add_slide_backward_left::<30>(self);
			iter.add_slide_backward_left::<31>(self);

			iter.add_slide_backward_right::<2>(self);
			iter.add_slide_backward_right::<3>(self);
			iter.add_slide_backward_right::<4>(self);
			iter.add_slide_backward_right::<5>(self);
			iter.add_slide_backward_right::<7>(self);
			iter.add_slide_backward_right::<8>(self);
			iter.add_slide_backward_right::<10>(self);
			iter.add_slide_backward_right::<11>(self);
			iter.add_slide_backward_right::<13>(self);
			iter.add_slide_backward_right::<14>(self);
			iter.add_slide_backward_right::<15>(self);
			iter.add_slide_backward_right::<16>(self);
			iter.add_slide_backward_right::<19>(self);
			iter.add_slide_backward_right::<20>(self);
			iter.add_slide_backward_right::<21>(self);
			iter.add_slide_backward_right::<22>(self);
			iter.add_slide_backward_right::<23>(self);
			iter.add_slide_backward_right::<24>(self);
			iter.add_slide_backward_right::<26>(self);
			iter.add_slide_backward_right::<27>(self);
			iter.add_slide_backward_right::<28>(self);
			iter.add_slide_backward_right::<29>(self);
			iter.add_slide_backward_right::<30>(self);
			iter.add_slide_backward_right::<31>(self);
		}

		iter
	}
}

impl PossibleMoves {
	// TODO test

	/// The highest possible number of valid moves
	pub const MAX_POSSIBLE_MOVES: usize = POSSIBLE_MOVES_ITER_SIZE;

	const fn slides_dark(board: CheckersBitBoard) -> Self {
		const FORWARD_LEFT_MASK: u32 = 0b01111001111110111111001111011011;
		const FORWARD_RIGHT_MASK: u32 = 0b01111101111111011111010111011101;
		const BACKWARD_LEFT_MASK: u32 = 0b11111011111110111110101110111010;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111001111110011110110110111100;

		let not_occupied = !board.pieces_bits();
		let friendly_pieces = board.pieces_bits() & board.color_bits();
		let friendly_kings = friendly_pieces & board.king_bits();

		let forward_left_movers =
			not_occupied.rotate_right(7) & friendly_pieces & FORWARD_LEFT_MASK;
		let forward_right_movers =
			not_occupied.rotate_right(1) & friendly_pieces & FORWARD_RIGHT_MASK;
		let backward_left_movers;
		let backward_right_movers;

		if friendly_kings > 0 {
			backward_left_movers =
				not_occupied.rotate_left(1) & friendly_kings & BACKWARD_LEFT_MASK;
			backward_right_movers =
				not_occupied.rotate_left(7) & friendly_kings & BACKWARD_RIGHT_MASK;
		} else {
			backward_left_movers = 0;
			backward_right_movers = 0;
		}

		Self {
			forward_left_movers,
			forward_right_movers,
			backward_left_movers,
			backward_right_movers,
		}
	}

	const fn slides_light(board: CheckersBitBoard) -> Self {
		const FORWARD_LEFT_MASK: u32 = 0b01111001111110111111001111011011;
		const FORWARD_RIGHT_MASK: u32 = 0b01111101111111011111010111011101;
		const BACKWARD_LEFT_MASK: u32 = 0b11111011111110111110101110111010;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111001111110011110110110111100;

		let not_occupied = !board.pieces_bits();
		let friendly_pieces = board.pieces_bits() & !board.color_bits();
		let friendly_kings = friendly_pieces & board.king_bits();

		let backward_left_movers =
			not_occupied.rotate_left(1) & friendly_pieces & BACKWARD_LEFT_MASK;
		let backward_right_movers =
			not_occupied.rotate_left(7) & friendly_pieces & BACKWARD_RIGHT_MASK;
		let forward_left_movers;
		let forward_right_movers;

		if friendly_kings > 0 {
			forward_left_movers = not_occupied.rotate_right(7) & friendly_kings & FORWARD_LEFT_MASK;
			forward_right_movers =
				not_occupied.rotate_right(1) & friendly_kings & FORWARD_RIGHT_MASK;
		} else {
			forward_left_movers = 0;
			forward_right_movers = 0;
		}

		Self {
			forward_left_movers,
			forward_right_movers,
			backward_left_movers,
			backward_right_movers,
		}
	}

	const fn jumps_dark(board: CheckersBitBoard) -> Self {
		const FORWARD_LEFT_MASK: u32 = 0b00110000111100111111001111000011;
		const FORWARD_RIGHT_MASK: u32 = 0b00111100111111001111000011001100;
		const BACKWARD_LEFT_MASK: u32 = 0b11110011111100111100001100110000;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111100111100001100110000111100;

		let not_occupied = !board.pieces_bits();
		let enemy_pieces = board.pieces_bits() & !board.color_bits();
		let friendly_pieces = board.pieces_bits() & board.color_bits();
		let friendly_kings = friendly_pieces & board.king_bits();

		let forward_left_movers = not_occupied.rotate_right(14)
			& enemy_pieces.rotate_right(7)
			& friendly_pieces
			& FORWARD_LEFT_MASK;
		let forward_right_movers = not_occupied.rotate_right(2)
			& enemy_pieces.rotate_right(1)
			& friendly_pieces
			& FORWARD_RIGHT_MASK;
		let backward_left_movers;
		let backward_right_movers;

		if friendly_kings > 0 {
			backward_left_movers = not_occupied.rotate_left(2)
				& enemy_pieces.rotate_left(1)
				& friendly_kings & BACKWARD_LEFT_MASK;
			backward_right_movers = not_occupied.rotate_left(14)
				& enemy_pieces.rotate_left(7)
				& friendly_kings & BACKWARD_RIGHT_MASK;
		} else {
			backward_left_movers = 0;
			backward_right_movers = 0;
		}

		let can_jump = if forward_left_movers != 0
			|| forward_right_movers != 0
			|| backward_left_movers != 0
			|| backward_right_movers != 0
		{
			2
		} else {
			0
		};

		Self {
			forward_left_movers,
			forward_right_movers,
			backward_left_movers,
			backward_right_movers: backward_right_movers | can_jump,
		}
	}

	const fn jumps_light(board: CheckersBitBoard) -> Self {
		const FORWARD_LEFT_MASK: u32 = 0b00110000111100111111001111000011;
		const FORWARD_RIGHT_MASK: u32 = 0b00111100111111001111000011001100;
		const BACKWARD_LEFT_MASK: u32 = 0b11110011111100111100001100110000;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111100111100001100110000111100;

		let not_occupied = !board.pieces_bits();
		let enemy_pieces = board.pieces_bits() & board.color_bits();
		let friendly_pieces = board.pieces_bits() & !board.color_bits();
		let friendly_kings = friendly_pieces & board.king_bits();

		let backward_left_movers = not_occupied.rotate_left(2)
			& enemy_pieces.rotate_left(1)
			& friendly_pieces
			& BACKWARD_LEFT_MASK;
		let backward_right_movers = not_occupied.rotate_left(14)
			& enemy_pieces.rotate_left(7)
			& friendly_pieces
			& BACKWARD_RIGHT_MASK;
		let forward_left_movers;
		let forward_right_movers;

		if friendly_kings > 0 {
			forward_left_movers = not_occupied.rotate_right(14)
				& enemy_pieces.rotate_right(7)
				& friendly_kings & FORWARD_LEFT_MASK;
			forward_right_movers = not_occupied.rotate_right(2)
				& enemy_pieces.rotate_right(1)
				& friendly_kings & FORWARD_RIGHT_MASK;
		} else {
			forward_left_movers = 0;
			forward_right_movers = 0;
		}

		let can_jump = if forward_left_movers != 0
			|| forward_right_movers != 0
			|| backward_left_movers != 0
			|| backward_right_movers != 0
		{
			2
		} else {
			0
		};

		Self {
			forward_left_movers,
			forward_right_movers,
			backward_left_movers,
			backward_right_movers: backward_right_movers | can_jump,
		}
	}

	const fn has_jumps_dark(board: CheckersBitBoard) -> bool {
		const FORWARD_LEFT_MASK: u32 = 0b00110000111100111111001111000011;
		const FORWARD_RIGHT_MASK: u32 = 0b00111100111111001111000011001100;
		const BACKWARD_LEFT_MASK: u32 = 0b11110011111100111100001100110000;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111100111100001100110000111100;

		let not_occupied = !board.pieces_bits();
		let enemy_pieces = board.pieces_bits() & !board.color_bits();
		let friendly_pieces = board.pieces_bits() & board.color_bits();

		let forward_left_spaces =
			not_occupied.rotate_right(14) & enemy_pieces.rotate_right(7) & FORWARD_LEFT_MASK;
		let forward_right_spaces =
			not_occupied.rotate_right(2) & enemy_pieces.rotate_right(1) & FORWARD_RIGHT_MASK;

		let forward_spaces = forward_left_spaces | forward_right_spaces;

		if board.king_bits() > 0 {
			let backward_left_spaces =
				not_occupied.rotate_left(2) & enemy_pieces.rotate_left(1) & BACKWARD_LEFT_MASK;
			let backward_right_spaces =
				not_occupied.rotate_left(14) & enemy_pieces.rotate_left(7) & BACKWARD_RIGHT_MASK;
			let backward_spaces = backward_left_spaces | backward_right_spaces;

			let backward_spaces = board.king_bits() & backward_spaces;
			friendly_pieces & (forward_spaces | backward_spaces) != 0
		} else {
			friendly_pieces & forward_spaces != 0
		}
	}

	const fn has_jumps_light(board: CheckersBitBoard) -> bool {
		const FORWARD_LEFT_MASK: u32 = 0b00110000111100111111001111000011;
		const FORWARD_RIGHT_MASK: u32 = 0b00111100111111001111000011001100;
		const BACKWARD_LEFT_MASK: u32 = 0b11110011111100111100001100110000;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111100111100001100110000111100;

		let not_occupied = !board.pieces_bits();
		let enemy_pieces = board.pieces_bits() & board.color_bits();
		let friendly_pieces = board.pieces_bits() & !board.color_bits();

		let backward_left_spaces =
			not_occupied.rotate_left(2) & enemy_pieces.rotate_left(1) & BACKWARD_LEFT_MASK;
		let backward_right_spaces =
			not_occupied.rotate_left(14) & enemy_pieces.rotate_left(7) & BACKWARD_RIGHT_MASK;

		let backward_spaces = backward_left_spaces | backward_right_spaces;

		if board.king_bits() > 0 {
			let forward_left_spaces =
				not_occupied.rotate_right(14) & enemy_pieces.rotate_right(7) & FORWARD_LEFT_MASK;
			let forward_right_spaces =
				not_occupied.rotate_right(2) & enemy_pieces.rotate_right(1) & FORWARD_RIGHT_MASK;
			let forward_spaces = forward_left_spaces | forward_right_spaces;

			let forward_spaces = board.king_bits() & forward_spaces;
			friendly_pieces & (forward_spaces | backward_spaces) != 0
		} else {
			friendly_pieces & backward_spaces != 0
		}
	}

	#[inline(always)]
	// TODO optimize
	pub const fn has_jumps(board: CheckersBitBoard) -> bool {
		match board.turn() {
			PieceColor::Light => Self::has_jumps_light(board),
			PieceColor::Dark => Self::has_jumps_dark(board),
		}
	}

	const fn has_jumps_at_dark(board: CheckersBitBoard, value: usize) -> bool {
		const FORWARD_LEFT_MASK: u32 = 0b00110000111100111111001111000011;
		const FORWARD_RIGHT_MASK: u32 = 0b00111100111111001111000011001100;
		const BACKWARD_LEFT_MASK: u32 = 0b11110011111100111100001100110000;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111100111100001100110000111100;

		let not_occupied = !board.pieces_bits();
		let enemy_pieces = board.pieces_bits() & !board.color_bits();
		let friendly_pieces = board.pieces_bits() & board.color_bits();

		let forward_left_spaces =
			not_occupied.rotate_right(14) & enemy_pieces.rotate_right(7) & FORWARD_LEFT_MASK;
		let forward_right_spaces =
			not_occupied.rotate_right(2) & enemy_pieces.rotate_right(1) & FORWARD_RIGHT_MASK;

		let forward_spaces = forward_left_spaces | forward_right_spaces;

		if board.king_bits() > 0 {
			let backward_left_spaces =
				not_occupied.rotate_left(2) & enemy_pieces.rotate_left(1) & BACKWARD_LEFT_MASK;
			let backward_right_spaces =
				not_occupied.rotate_left(14) & enemy_pieces.rotate_left(7) & BACKWARD_RIGHT_MASK;
			let backward_spaces = backward_left_spaces | backward_right_spaces;

			let backward_spaces = board.king_bits() & backward_spaces;
			((friendly_pieces & (forward_spaces | backward_spaces)) >> value) & 1 != 0
		} else {
			((friendly_pieces & forward_spaces) >> value) & 1 != 0
		}
	}

	const fn has_jumps_at_light(board: CheckersBitBoard, value: usize) -> bool {
		const FORWARD_LEFT_MASK: u32 = 0b00110000111100111111001111000011;
		const FORWARD_RIGHT_MASK: u32 = 0b00111100111111001111000011001100;
		const BACKWARD_LEFT_MASK: u32 = 0b11110011111100111100001100110000;
		const BACKWARD_RIGHT_MASK: u32 = 0b11111100111100001100110000111100;

		let not_occupied = !board.pieces_bits();
		let enemy_pieces = board.pieces_bits() & board.color_bits();
		let friendly_pieces = board.pieces_bits() & !board.color_bits();

		let backward_left_spaces =
			not_occupied.rotate_left(2) & enemy_pieces.rotate_left(1) & BACKWARD_LEFT_MASK;
		let backward_right_spaces =
			not_occupied.rotate_left(14) & enemy_pieces.rotate_left(7) & BACKWARD_RIGHT_MASK;

		let backward_spaces = backward_left_spaces | backward_right_spaces;

		if board.king_bits() > 0 {
			let forward_left_spaces =
				not_occupied.rotate_right(14) & enemy_pieces.rotate_right(7) & FORWARD_LEFT_MASK;
			let forward_right_spaces =
				not_occupied.rotate_right(2) & enemy_pieces.rotate_right(1) & FORWARD_RIGHT_MASK;
			let forward_spaces = forward_left_spaces | forward_right_spaces;

			let forward_spaces = board.king_bits() & forward_spaces;
			((friendly_pieces & (forward_spaces | backward_spaces)) >> value) & 1 != 0
		} else {
			((friendly_pieces & backward_spaces) >> value) & 1 != 0
		}
	}

	#[inline(always)]
	// TODO optimize
	pub const fn has_jumps_at(board: CheckersBitBoard, value: usize) -> bool {
		match board.turn() {
			PieceColor::Light => Self::has_jumps_at_light(board, value),
			PieceColor::Dark => Self::has_jumps_at_dark(board, value),
		}
	}

	const fn light_moves(board: CheckersBitBoard) -> Self {
		let jumps = Self::jumps_light(board);
		if jumps.is_empty() {
			Self::slides_light(board)
		} else {
			jumps
		}
	}

	const fn dark_moves(board: CheckersBitBoard) -> Self {
		let jumps = Self::jumps_dark(board);
		if jumps.is_empty() {
			Self::slides_dark(board)
		} else {
			jumps
		}
	}

	pub const fn moves(board: CheckersBitBoard) -> Self {
		match board.turn() {
			PieceColor::Dark => Self::dark_moves(board),
			PieceColor::Light => Self::light_moves(board),
		}
	}

	/// Returns true if no moves are possible
	pub const fn is_empty(self) -> bool {
		(self.backward_left_movers
			| (self.forward_left_movers)
			| self.forward_right_movers
			| self.backward_right_movers & 4294967293)
			== 0
	}

	/// Returns true if the piece can jump
	pub const fn can_jump(self) -> bool {
		(self.backward_right_movers & 2) != 0
	}

	/// Returns true if the given move is possible
	pub const fn contains(self, checker_move: Move) -> bool {
		if checker_move.is_jump() != self.can_jump() {
			return false;
		}

		let bits = match checker_move.direction() {
			MoveDirection::ForwardLeft => self.forward_left_movers,
			MoveDirection::ForwardRight => self.forward_right_movers,
			MoveDirection::BackwardLeft => self.backward_left_movers,
			MoveDirection::BackwardRight => self.backward_right_movers,
		};

		(bits >> checker_move.start()) & 1 == 1
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn setup_empty_iter() -> PossibleMovesIter {
		let moves = [MaybeUninit::uninit(); POSSIBLE_MOVES_ITER_SIZE];
		PossibleMovesIter {
			moves,
			index: 0,
			length: 0,
		}
	}

	fn setup_add_move_to_iter_invalid() -> (PossibleMovesIter, PossibleMoves) {
		let moves = PossibleMoves {
			forward_left_movers: 0,
			forward_right_movers: 0,
			backward_left_movers: 0,
			backward_right_movers: 0,
		};
		let iter = setup_empty_iter();

		(iter, moves)
	}

	fn setup_add_move_to_iter_valid() -> (PossibleMovesIter, PossibleMoves) {
		let moves = PossibleMoves {
			forward_left_movers: u32::MAX,
			forward_right_movers: u32::MAX,
			backward_left_movers: u32::MAX,
			backward_right_movers: u32::MAX,
		};
		let iter = setup_empty_iter();

		(iter, moves)
	}

	#[test]
	fn same() {
		let start = CheckersBitBoard::new(
			0b11100111100111100111110111111011,
			0b00001100001111001111001111000011,
			0,
			PieceColor::Dark,
		);
		let flip = CheckersBitBoard::new(
			0b11100111100111100111110111111011,
			0b11110011110000110000110000111100,
			0,
			PieceColor::Light,
		);

		assert_eq!(
			PossibleMoves::has_jumps(start),
			PossibleMoves::has_jumps(flip)
		)
	}

	#[test]
	fn iter_next() {
		let test_move1 = Move::new(8, MoveDirection::ForwardLeft, false);
		let test_move2 = Move::new(26, MoveDirection::ForwardRight, true);
		let mut iter = setup_empty_iter();
		iter.length = 2;

		let ptr = iter.moves.as_mut().get_mut(0).unwrap();
		*ptr = MaybeUninit::new(test_move1);

		let ptr = iter.moves.as_mut().get_mut(1).unwrap();
		*ptr = MaybeUninit::new(test_move2);

		let recieved_move = iter.next();
		assert!(recieved_move.is_some());
		assert_eq!(recieved_move.unwrap(), test_move1);

		let recieved_move = iter.next();
		assert!(recieved_move.is_some());
		assert_eq!(recieved_move.unwrap(), test_move2);

		let recieved_move = iter.next();
		assert!(recieved_move.is_none());
	}

	#[test]
	fn add_slide_forward_left_to_iter_invalid() {
		const START: usize = 8;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_slide_forward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_slide_forward_left_to_iter_valid() {
		const START: usize = 8;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_slide_forward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::ForwardLeft);
		assert!(!new_move.is_jump());
	}

	#[test]
	fn add_slide_forward_right_to_iter_invalid() {
		const START: usize = 26;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_slide_forward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_slide_forward_right_to_iter_valid() {
		const START: usize = 26;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_slide_forward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::ForwardRight);
		assert!(!new_move.is_jump());
	}

	#[test]
	fn add_slide_backward_left_to_iter_invalid() {
		const START: usize = 17;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_slide_backward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_slide_backward_left_to_iter_valid() {
		const START: usize = 17;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_slide_backward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::BackwardLeft);
		assert!(!new_move.is_jump());
	}

	#[test]
	fn add_slide_backward_right_to_iter_invalid() {
		const START: usize = 3;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_slide_backward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_slide_backward_right_to_iter_valid() {
		const START: usize = 3;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_slide_backward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::BackwardRight);
		assert!(!new_move.is_jump());
	}

	#[test]
	fn add_jump_forward_left_to_iter_invalid() {
		const START: usize = 8;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_jump_forward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_jump_forward_left_to_iter_valid() {
		const START: usize = 8;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_jump_forward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::ForwardLeft);
		assert!(new_move.is_jump());
	}

	#[test]
	fn add_jump_forward_right_to_iter_invalid() {
		const START: usize = 26;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_jump_forward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_jump_forward_right_to_iter_valid() {
		const START: usize = 26;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_jump_forward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::ForwardRight);
		assert!(new_move.is_jump());
	}

	#[test]
	fn add_jump_backward_left_to_iter_invalid() {
		const START: usize = 17;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_jump_backward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_jump_backward_left_to_iter_valid() {
		const START: usize = 17;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_jump_backward_left::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::BackwardLeft);
		assert!(new_move.is_jump());
	}

	#[test]
	fn add_jump_backward_right_to_iter_invalid() {
		const START: usize = 3;
		let (mut iter, moves) = setup_add_move_to_iter_invalid();
		iter.add_jump_backward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 0);
	}

	#[test]
	fn add_jump_backward_right_to_iter_valid() {
		const START: usize = 3;
		let (mut iter, moves) = setup_add_move_to_iter_valid();
		iter.add_jump_backward_right::<START>(moves);

		assert_eq!(iter.index, 0);
		assert_eq!(iter.length, 1);

		let new_move = iter.next().unwrap();
		assert_eq!(new_move.start(), START as u32);
		assert_eq!(new_move.direction(), MoveDirection::BackwardRight);
		assert!(new_move.is_jump());
	}

	#[test]
	fn cant_jump_in_position_2_without_26() {
		// This bug was bizarre, but it's caused by a white piece being in the
		//second bit while there is no piece in the 26th bit. If you don't
		// apply the bit mask for collision detection, then all of the light
		// player moves become jumps.
		let board = CheckersBitBoard {
			pieces: 16908890,
			color: 401395713,
			kings: 50332352,
			turn: PieceColor::Light,
		};
		let possible_moves = PossibleMoves::moves(board);
		assert!(!possible_moves.can_jump())
	}

	#[test]
	fn not_has_jump_at_14_when_has_jump_at_20() {
		// This bug was caused by me forgetting to `& 1` to the end of the
		// `has_jump_at` functions. After playing a jump with one piece, I was
		// able to continue jumping with completely different pieces
		let board = CheckersBitBoard::new(
			0b11100111001111001111110111111011,
			0b00001100001111001111001111000011,
			0,
			PieceColor::Dark,
		);
		let possible_moves = PossibleMoves::moves(board);
		assert!(!possible_moves.can_jump())
	}

	#[test]
	fn test_send() {
		fn assert_send<T: Send>() {}
		assert_send::<PossibleMoves>();
		assert_send::<PossibleMovesIter>();
	}

	#[test]
	fn test_sync() {
		fn assert_sync<T: Sync>() {}
		assert_sync::<PossibleMoves>();
		assert_sync::<PossibleMovesIter>();
	}
}
