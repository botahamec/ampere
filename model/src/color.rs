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
	/// Flips the color
	pub const fn flip(self) -> Self {
		// TODO optimize
		match self {
			PieceColor::Light => PieceColor::Dark,
			PieceColor::Dark => PieceColor::Light,
		}
	}

	/// Flips the color if the statement is true
	///
	/// # Arguments
	///
	/// * `statement` - Flips the color if true
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

	#[test]
	fn flip() {
		let light = PieceColor::Light;
		let dark = PieceColor::Dark;
		assert_eq!(light.flip(), dark);
		assert_eq!(dark.flip(), light);
	}

	#[test]
	fn flip_if() {
		let light = PieceColor::Light;
		let dark = PieceColor::Dark;

		assert_eq!(light.flip_if(true), dark);
		assert_eq!(light.flip_if(false), light);
		assert_eq!(dark.flip_if(true), light);
		assert_eq!(dark.flip_if(false), dark);
	}

	#[test]
	fn test_send() {
		fn assert_send<T: Send>() {}
		assert_send::<PieceColor>();
	}

	#[test]
	fn test_sync() {
		fn assert_sync<T: Sync>() {}
		assert_sync::<PieceColor>();
	}
}
