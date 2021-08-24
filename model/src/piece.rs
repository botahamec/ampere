use crate::PieceColor;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Piece {
	king: bool,
	color: PieceColor,
}

impl Piece {
	pub(crate) const fn new(king: bool, color: PieceColor) -> Self {
		Self { king, color }
	}

	pub const fn is_king(self) -> bool {
		self.king
	}

	pub const fn color(self) -> PieceColor {
		self.color
	}
}
