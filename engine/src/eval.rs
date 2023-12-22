use std::fmt::{self, Display};
use std::ops::Neg;

use model::CheckersBitBoard;

const KING_WORTH: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Evaluation(i16);

impl Display for Evaluation {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.is_force_win() {
			write!(f, "+M{}", self.force_sequence_length().unwrap())
		} else if self.is_force_loss() {
			write!(f, "-M{}", self.force_sequence_length().unwrap())
		} else {
			write!(f, "{:+}", self.to_f32().unwrap())
		}
	}
}

impl Neg for Evaluation {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self(-self.0)
	}
}

impl Evaluation {
	pub(crate) const NULL_MAX: Self = Self(i16::MAX);
	pub(crate) const NULL_MIN: Self = Self(i16::MIN + 1);

	pub const WIN: Self = Self(i16::MAX - 1);
	pub const DRAW: Self = Self(0);
	pub const LOSS: Self = Self(i16::MIN + 2);

	// last fourteen bits set to 1
	const FORCE_WIN_THRESHOLD: i16 = 0x3FFF;

	pub fn new(eval: f32) -> Self {
		if eval >= 1.0 {
			return Self::WIN;
		} else if eval <= -1.0 {
			return Self::LOSS;
		}

		Self((eval * 16384.0) as i16)
	}

	pub fn to_f32(self) -> Option<f32> {
		if self.is_force_sequence() {
			return None;
		}

		Some(self.0 as f32 / 16384.0)
	}

	pub fn is_force_win(self) -> bool {
		self.0 > Self::FORCE_WIN_THRESHOLD
	}

	pub fn is_force_loss(self) -> bool {
		self.0 < -Self::FORCE_WIN_THRESHOLD
	}

	pub fn is_force_sequence(self) -> bool {
		self.is_force_win() || self.is_force_loss()
	}

	pub fn force_sequence_length(self) -> Option<u8> {
		if self == Self::NULL_MAX || self == Self::NULL_MIN {
			return None;
		}

		if self.is_force_win() {
			Some((Self::WIN.0 - self.0) as u8)
		} else if self.is_force_loss() {
			Some((self.0 - Self::LOSS.0) as u8)
		} else {
			None
		}
	}

	pub fn increment(self) -> Self {
		if self.is_force_win() {
			Self(self.0 - 1)
		} else if self.is_force_loss() {
			Self(self.0 + 1)
		} else {
			self
		}
	}

	pub fn add(self, rhs: f32) -> Self {
		let Some(eval) = self.to_f32() else {
			return self;
		};

		Self::new(eval + rhs)
	}
}

pub fn eval_position(board: CheckersBitBoard) -> Evaluation {
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
		Evaluation::new((dark_eval - light_eval) / (dark_eval + light_eval))
	} else {
		Evaluation::DRAW
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn zero_eval() {
		let draw = Evaluation::new(0.0);
		assert_eq!(draw, Evaluation::DRAW);
		assert_eq!(draw.to_f32(), Some(0.0));
		assert_eq!(draw.to_string(), "+0");
	}

	#[test]
	fn comparisons() {
		assert!(Evaluation::NULL_MAX > Evaluation::WIN);
		assert!(Evaluation::WIN > Evaluation::new(0.5));
		assert!(Evaluation::new(0.5) > Evaluation::DRAW);
		assert!(Evaluation::DRAW > Evaluation::new(-0.5));
		assert!(Evaluation::new(-0.5) > Evaluation::LOSS);
		assert!(Evaluation::LOSS > Evaluation::NULL_MIN);
	}

	#[test]
	fn negations() {
		assert_eq!(-Evaluation::NULL_MAX, Evaluation::NULL_MIN);
		assert_eq!(-Evaluation::NULL_MIN, Evaluation::NULL_MAX);
		assert_eq!(-Evaluation::WIN, Evaluation::LOSS);
		assert_eq!(-Evaluation::LOSS, Evaluation::WIN);
		assert_eq!(-Evaluation::DRAW, Evaluation::DRAW);
		assert_eq!(-Evaluation::new(0.5), Evaluation::new(-0.5));
	}
}
