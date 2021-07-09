#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The color of a piece
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PieceColor {
	Light,
	Dark,
}

impl Display for PieceColor {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Light => "Light",
				Self::Dark => "Dark",
			}
		)
	}
}

impl PieceColor {
	pub const fn flip(self) -> Self {
		// TODO optimize
		match self {
			PieceColor::Light => PieceColor::Dark,
			PieceColor::Dark => PieceColor::Light,
		}
	}

	pub const fn flip_if(self, statement: bool) -> Self {
		if statement {
			self.flip()
		} else {
			self
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn light_display() {
		assert_eq!(PieceColor::Light.to_string(), "Light");
	}

	#[test]
	fn dark_display() {
		assert_eq!(PieceColor::Dark.to_string(), "Dark");
	}
}
