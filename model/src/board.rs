use crate::possible_moves::PossibleMoves;
use crate::{Piece, PieceColor, SquareCoordinate};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[cfg(test)]
mod tests;

/// A checker board,
/// organized in the following structure:
/// ```txt
///   11  05  31  25
/// 10  04  30  24
///   03  29  23  17
/// 02  28  22  16
///   27  21  15  09
/// 26  20  14  08
///   19  13  07  01
/// 18  12  06  00
/// ```
#[derive(Copy, Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CheckersBitBoard {
	/// If the space contains a piece, it's a 1
	pieces: u32,
	/// If the piece is black, 1, otherwise 0
	color: u32,
	/// 1 if the piece is a king
	kings: u32,
	/// The player who has the next turn
	turn: PieceColor,
}

impl Default for CheckersBitBoard {
	/// Returns the starting position
	fn default() -> Self {
		Self::starting_position()
	}
}

impl PartialEq for CheckersBitBoard {
	fn eq(&self, other: &Self) -> bool {
		self.pieces == other.pieces
			&& self.pieces & self.color == other.pieces & other.color
			&& self.pieces & self.kings == other.pieces & other.kings
			&& self.turn == other.turn
	}
}

impl Hash for CheckersBitBoard {
	/// Hashes with only the pieces part, to ensure correctness and efficiency
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.pieces.hash(hasher)
	}
}

impl CheckersBitBoard {
	/// Creates a new Checkers BitBoard
	///
	/// # Arguments
	///
	/// * `pieces` - Each bit is 1 if the corresponding space contains a piece
	/// * `color` - For each space with a piece, the value is 1 if it's dark, and 0 otherwise.
	/// Bits for spaces without colors are undefined
	/// * `kings` - For each space with a piece, the value is 1 if it's a king, and 0 otherwise.
	/// Bits for spaces without colors are undefined
	///
	/// # Example
	///
	/// ```
	/// // This is the starting position
	/// use model::{CheckersBitBoard, PieceColor};
	/// let board = CheckersBitBoard::new(0b11011111101111100111100111100111,
	///                                   0b00111100001100001100001111001111,
	///                                   0,
	///                                   PieceColor::Dark);
	/// ```
	pub const fn new(pieces: u32, color: u32, kings: u32, turn: PieceColor) -> Self {
		Self {
			pieces,
			color,
			kings,
			turn,
		}
	}

	/// Creates a board at the starting position
	pub const fn starting_position() -> Self {
		const STARTING_BITBOARD: CheckersBitBoard = CheckersBitBoard::new(
			0b11100111100111100111110111111011,
			0b00001100001111001111001111000011,
			0,
			PieceColor::Dark,
		);
		STARTING_BITBOARD
	}

	/// Gets the bits that represent where pieces are on the board
	pub const fn pieces_bits(self) -> u32 {
		self.pieces
	}

	/// Gets the bits that represents the color of each piece on the board
	///
	/// # Safety
	///
	/// This is inherently unsafe, because this also returns the bits of empty squares
	pub const fn color_bits(self) -> u32 {
		self.color
	}

	/// Gets the bits that represents the status of each piece on the board
	///
	/// # Safety
	///
	/// This is inherently unsafe, because this also returns the bits of empty squares
	pub const fn king_bits(self) -> u32 {
		self.kings
	}

	/// The player whose turn it is
	pub const fn turn(self) -> PieceColor {
		self.turn
	}

	/// Gets the piece at a given row column coordinate
	///
	/// # Arguments
	///
	/// * `row` - The row. The a file is row 0
	/// * `col` - The column. The first rank is column 0
	pub fn get_at_row_col(self, row: usize, col: usize) -> Option<Piece> {
		if row > 32 || col > 32 {
			None
		} else {
			let value = SquareCoordinate::new(row as u8, col as u8).to_value();
			if let Some(value) = value {
				if self.piece_at(value) {
					Some(Piece::new(
						unsafe { self.king_at_unchecked(value) },
						unsafe { self.color_at_unchecked(value) },
					))
				} else {
					None
				}
			} else {
				None
			}
		}
	}

	/// Checks if there's a piece at the given space value
	///
	/// # Arguments
	///
	/// * `value` - The value of the space to check
	///
	/// # Example
	///
	/// ```
	/// use model::CheckersBitBoard;
	/// let board = CheckersBitBoard::default();
	/// match board.piece_at(0) {
	///     true => println!("There's a piece in the bottom right"),
	///     false => println!("The bottom right is empty")
	/// }
	/// ```
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	pub const fn piece_at(self, value: usize) -> bool {
		((self.pieces >> value) & 1) == 1
	}

	/// Checks the color at the piece in the given location,
	/// without checking if there's a piece there
	///
	/// # Arguments
	///
	/// * `value` - The value of the space to check
	///
	/// # Example
	///
	/// ```
	/// use model::CheckersBitBoard;
	/// use model::PieceColor;
	/// let board = CheckersBitBoard::default();
	/// if board.piece_at(0) {
	///     match unsafe {board.color_at_unchecked(0)} {
	///         PieceColor::Dark => println!("The piece in the bottom right is dark colored"),
	///         PieceColor::Light => println!("The piece in the bottom right is light colored")
	///     }
	/// }
	/// ```
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Checking the color at a square that is empty results in undefined behavior
	pub const unsafe fn color_at_unchecked(self, value: usize) -> PieceColor {
		if ((self.color >> value) & 1) != 0 {
			PieceColor::Dark
		} else {
			PieceColor::Light
		}
	}

	/// Checks the color at the piece in the given location.
	/// Returns `None` if there isn't a piece there
	///
	/// # Arguments
	///
	/// * `value` - The value of the space to check
	///
	/// # Example
	///
	/// ```
	/// use model::CheckersBitBoard;
	/// use model::PieceColor;
	/// let board = CheckersBitBoard::default();
	/// if let Some(color) = board.color_at(0) {
	///     match color {
	///         PieceColor::Dark => println!("The piece in the bottom right is dark colored"),
	///         PieceColor::Light => println!("The piece in the bottom left is light colored")
	///     }
	/// }
	/// ```
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	pub const fn color_at(self, value: usize) -> Option<PieceColor> {
		if self.piece_at(value) {
			// safety: if this block runs, then it's already confirmed a piece exists here
			Some(unsafe { self.color_at_unchecked(value) })
		} else {
			None
		}
	}

	/// Checks if the given location has a king, without checking if there's a piece there
	///
	/// # Arguments
	///
	/// * `value` - The value of the space to check
	///
	/// # Example
	///
	/// ```
	/// use model::CheckersBitBoard;
	/// let board = CheckersBitBoard::default();
	/// if board.piece_at(0) {
	///     match unsafe {board.king_at_unchecked(0)} {
	///         true => println!("The piece in the bottom right is a king"),
	///         false => println!("The piece in the bottom right is a peasant")
	///     }
	/// }
	/// ```
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Checking a square that is empty results in undefined behavior
	pub const unsafe fn king_at_unchecked(self, value: usize) -> bool {
		((self.kings >> value) & 1) == 1
	}

	/// Checks if the piece in the given location is a king.
	/// Returns `None` if there isn't a piece there
	///
	/// # Arguments
	///
	/// * `value` - The value of the space to check
	///
	/// # Example
	///
	/// ```
	/// use model::CheckersBitBoard;
	/// let board = CheckersBitBoard::default();
	/// if let Some(status) = board.king_at(0) {
	///     match status {
	///         true => println!("The piece in the bottom right is a king"),
	///         false => println!("The piece in the bottom right is a peasant")
	///     }
	/// }
	/// ```
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	pub const fn king_at(self, value: usize) -> Option<bool> {
		if self.piece_at(value) {
			// safety: if this block runs, then it's already confirmed a piece exists here
			Some(unsafe { self.king_at_unchecked(value) })
		} else {
			None
		}
	}

	pub const fn flip_turn(self) -> Self {
		CheckersBitBoard::new(self.pieces, self.color, self.kings, self.turn.flip())
	}

	/// Moves a piece from `start` to `dest`. The original location will be empty.
	/// This does not mutate the original board.
	/// If a piece already exists at `dest`, it will be overwritten.
	///
	/// # Arguments
	///
	/// * `start` - The original location of the piece
	/// * `dest` - The new location
	///
	/// # Panics
	///
	/// Panics if `start` or `dest` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Results in undefined behavior if `start` does not contain a piece
	// TODO rip out so we don't need to check for both black and white promotion
	pub const unsafe fn move_piece_to_unchecked(self, start: usize, dest: usize) -> Self {
		// Clears the bit at the starting value
		// Sets the bit at the destination value
		let pieces = (self.pieces & !(1 << start)) | (1 << dest);

		// Clears the bit at the destination value
		// Sets the value at the destination to the value of the start
		let color = (self.color & !(1 << dest)) | (((self.color >> start) & 1) << dest);

		// The squares where certain pieces should be promoted
		const DARK_PROMOTION_MASK: u32 = 0b10000010000000000000100000100000;
		const LIGHT_PROMOTION_MASK: u32 = 0b1000001000001000001;

		// Clears the bit at the destination value
		// Sets the value at the destination to the value of the start
		// Promotes if the end of the board was reached
		let kings = (self.kings & !(1 << dest))
			| (((self.kings >> start) & 1) << dest)
			| (color & DARK_PROMOTION_MASK)
			| (!color & LIGHT_PROMOTION_MASK);

		let turn = self.turn.flip();

		CheckersBitBoard::new(pieces, color, kings, turn)
	}

	/// Moves a piece from `value` to `(value + amount) % 32`. The original location will be empty.
	/// This does not mutate the original board
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	/// * `amount` - The amount to shift the location by
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32,
	/// or `value + amount` is greater than `usize::MAX`
	///
	/// # Safety
	///
	/// This results in undefined behavior if `value` does not contain a piece
	const unsafe fn move_piece_forward_unchecked(self, value: usize, amount: usize) -> Self {
		self.move_piece_to_unchecked(value, (value + amount) & 31)
	}

	/// Moves a piece from `value` to `(value - amount) % 32`. The original location will be empty.
	/// This does not mutate the original board.
	/// If a piece already exists there, then it will be overwritten
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	/// * `amount` - The amount to shift the location by
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// This results in undefined behavior if `value` does not contain a piece
	const unsafe fn move_piece_backward_unchecked(self, value: usize, amount: usize) -> Self {
		self.move_piece_to_unchecked(value, value.wrapping_sub(amount) & 31)
	}

	/// Tries to move the piece forward and to the left, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the left side of the board results in undefined behavior.
	/// Moving from the top of the board results in undefined behavior.
	/// A `value` which doesn't contain a piece results in undefined behavior.
	pub const unsafe fn move_piece_forward_left_unchecked(self, value: usize) -> Self {
		self.move_piece_forward_unchecked(value, 7)
	}

	/// Tries to move the piece forward and to the right, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the right side of the board results in undefined behavior.
	/// Moving from the top of the board results in undefined behavior.
	/// A `value` which doesn't contain a piece results in undefined behavior.
	pub const unsafe fn move_piece_forward_right_unchecked(self, value: usize) -> Self {
		self.move_piece_forward_unchecked(value, 1)
	}

	/// Tries to move the piece backward and to the left, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the left side of the board results in undefined behavior.
	/// Moving from the bottom of the board results in undefined behavior.
	/// A `value` which doesn't contain a piece results in undefined behavior.
	pub const unsafe fn move_piece_backward_left_unchecked(self, value: usize) -> Self {
		self.move_piece_backward_unchecked(value, 1)
	}

	/// Tries to move the piece backward and to the right, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the right side of the board results in undefined behavior.
	/// Moving from the bottom of the board results in undefined behavior.
	/// A `value` which doesn't contain a piece results in undefined behavior.
	pub const unsafe fn move_piece_backward_right_unchecked(self, value: usize) -> Self {
		self.move_piece_backward_unchecked(value, 7)
	}

	/// Clears a space on the board. If the space is empty, then this function does nothing.
	///
	/// # Arguments
	///
	/// * `value` - The value of the space to clear
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	pub const fn clear_piece(self, value: usize) -> Self {
		let pieces = self.pieces & !(1 << value);
		CheckersBitBoard::new(pieces, self.color, self.kings, self.turn)
	}

	/// Tries to jump the piece forward and to the left, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten.
	/// The space the piece jumps over is cleared
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the left side of the board results in undefined behavior.
	/// Moving from the top of the board results in undefined behavior
	pub const unsafe fn jump_piece_forward_left_unchecked(self, value: usize) -> Self {
		let not_king = !self.king_at_unchecked(value);
		let board = self
			.move_piece_forward_unchecked(value, 14)
			.clear_piece((value + 7) & 31);

		const KING_MASK: u32 = 0b01000001000000000000010000010000;
		if PossibleMoves::has_jumps(board.flip_turn())
			&& not_king && (((1 << value) & KING_MASK) == 0)
		{
			board.flip_turn()
		} else {
			board
		}
	}

	/// Tries to move the piece forward and to the right, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten
	/// The space the piece jumps over is cleared
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the right side of the board results in undefined behavior.
	/// Moving from the top of the board results in undefined behavior
	pub const unsafe fn jump_piece_forward_right_unchecked(self, value: usize) -> Self {
		let not_king = !self.king_at_unchecked(value);
		let board = self
			.move_piece_forward_unchecked(value, 2)
			.clear_piece((value + 1) & 31);

		const KING_MASK: u32 = 0b01000001000000000000010000010000;
		if PossibleMoves::has_jumps(board.flip_turn())
			&& not_king && (((1 << value) & KING_MASK) == 0)
		{
			board.flip_turn()
		} else {
			board
		}
	}

	/// Tries to move the piece backward and to the left, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten
	/// The space the piece jumps over is cleared
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the left side of the board results in undefined behavior.
	/// Moving from the bottom of the board results in undefined behavior
	pub const unsafe fn jump_piece_backward_left_unchecked(self, value: usize) -> Self {
		let not_king = !self.king_at_unchecked(value);
		let board = self
			.move_piece_backward_unchecked(value, 2)
			.clear_piece(value.wrapping_sub(1) & 31);

		const KING_MASK: u32 = 0b00000000000010000010000010000010;
		if PossibleMoves::has_jumps(board.flip_turn())
			&& not_king && (((1 << value) & KING_MASK) == 0)
		{
			board.flip_turn()
		} else {
			board
		}
	}

	/// Tries to move the piece backward and to the right, without checking if it's a legal move.
	/// If a piece already exists there, then it will be overwritten
	/// The space the piece jumps over is cleared
	///
	/// # Arguments
	///
	/// * `value` - The original location of the piece
	///
	/// # Panics
	///
	/// Panics if `value` is greater than or equal to 32
	///
	/// # Safety
	///
	/// Moving from the right side of the board results in undefined behavior.
	/// Moving from the bottom of the board results in undefined behavior
	pub const unsafe fn jump_piece_backward_right_unchecked(self, value: usize) -> Self {
		let not_king = !self.king_at_unchecked(value);
		let board = self
			.move_piece_backward_unchecked(value, 14)
			.clear_piece(value.wrapping_sub(7) & 31);

		const KING_MASK: u32 = 0b00000000000010000010000010000010;
		if PossibleMoves::has_jumps(board.flip_turn())
			&& not_king && (((1 << value) & KING_MASK) == 0)
		{
			board.flip_turn()
		} else {
			board
		}
	}
}
