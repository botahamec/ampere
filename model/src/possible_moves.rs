use crate::moves::{Move, MoveDirection};
use crate::{CheckersBitBoard, PieceColor};
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::mem::MaybeUninit;
use std::ptr::NonNull;

const POSSIBLE_MOVES_ITER_SIZE: usize = 42;

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
	moves: NonNull<[MaybeUninit<Move>; POSSIBLE_MOVES_ITER_SIZE]>,

	/// The current index into the moves array
	index: usize,

	// The number of initialized moves in the array
	length: usize,
}

impl PossibleMovesIter {
	fn add_slide_forward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_left_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardLeft, false));
			self.length += 1;
		}
	}

	fn add_slide_forward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_right_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardRight, false));
			self.length += 1;
		}
	}

	fn add_slide_backward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_left_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardLeft, false));
			self.length += 1;
		}
	}

	fn add_slide_backward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_right_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardRight, false));
			self.length += 1;
		}
	}

	fn add_jump_forward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_left_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardLeft, true));
			self.length += 1;
		}
	}

	fn add_jump_forward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.forward_right_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::ForwardRight, true));
			self.length += 1;
		}
	}

	fn add_jump_backward_left<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_left_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardLeft, true));
			self.length += 1;
		}
	}

	fn add_jump_backward_right<const SQUARE: usize>(&mut self, possible_moves: PossibleMoves) {
		if (possible_moves.backward_right_movers >> SQUARE) & 1 != 0 {
			let ptr = unsafe { self.moves.as_mut().get_unchecked_mut(self.length) };
			*ptr = MaybeUninit::new(Move::new(SQUARE, MoveDirection::BackwardRight, true));
			self.length += 1;
		}
	}
}

impl Iterator for PossibleMovesIter {
	type Item = Move;

	fn next(&mut self) -> Option<Self::Item> {
		if self.length != self.index {
			let next_move = unsafe { self.moves.as_ref().get_unchecked(self.index).assume_init() };
			self.index += 1;
			Some(next_move)
		} else {
			None
		}
	}
}

impl Drop for PossibleMovesIter {
	fn drop(&mut self) {
		let layout = Layout::array::<MaybeUninit<Move>>(POSSIBLE_MOVES_ITER_SIZE).unwrap();
		unsafe { dealloc(self.moves.as_ptr() as *mut u8, layout) }
	}
}

impl IntoIterator for PossibleMoves {
	type Item = Move;
	type IntoIter = PossibleMovesIter;

	fn into_iter(self) -> Self::IntoIter {
		let layout = Layout::array::<MaybeUninit<Move>>(POSSIBLE_MOVES_ITER_SIZE).unwrap();
		let allocated_mem = unsafe { alloc(layout) };
		let ptr =
			match NonNull::new(allocated_mem as *mut [MaybeUninit<Move>; POSSIBLE_MOVES_ITER_SIZE])
			{
				Some(p) => p,
				None => handle_alloc_error(layout),
			};
		let mut iter = PossibleMovesIter {
			moves: ptr,
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

		Self {
			forward_left_movers,
			forward_right_movers,
			backward_left_movers,
			backward_right_movers: backward_right_movers | 2,
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

		Self {
			forward_left_movers,
			forward_right_movers,
			backward_left_movers,
			backward_right_movers: backward_right_movers | 2,
		}
	}

	// TODO make this private
	pub const fn has_jumps_dark(board: CheckersBitBoard) -> bool {
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

	// TODO make this private
	pub const fn has_jumps_light(board: CheckersBitBoard) -> bool {
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

	/// Returns the pieces who can move forward left,
	/// with undefined behavior for bits where a forward left move is impossible
	///
	/// # Safety
	///
	/// This function is inherently unsafe because some bits are undefined
	// TODO make this unsafe
	pub const fn forward_left_bits(self) -> u32 {
		self.forward_left_movers
	}

	/// Gets the bits for a certain direction,
	/// with undefined behavior for bits where the given move is impossible
	///
	/// # Safety
	///
	/// This function is inherently unsafe because some bits are undefined
	// TODO make this unsafe
	pub const fn get_direction_bits(self, direction: MoveDirection) -> u32 {
		match direction {
			MoveDirection::ForwardLeft => self.forward_left_movers,
			MoveDirection::ForwardRight => self.forward_right_movers,
			MoveDirection::BackwardLeft => self.backward_left_movers,
			MoveDirection::BackwardRight => self.backward_right_movers,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

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
}
