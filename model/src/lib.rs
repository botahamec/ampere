mod board;
mod color;
mod coordinates;
mod moves;
mod piece;
mod possible_moves;

pub use board::CheckersBitBoard;
pub use color::PieceColor;
pub use coordinates::SquareCoordinate;
pub use moves::Move;
pub use piece::Piece;
pub use possible_moves::PossibleMoves;
