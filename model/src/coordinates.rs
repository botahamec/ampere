use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SquareCoordinate {
	rank: u8,
	file: u8,
}

impl SquareCoordinate {
	pub fn new(rank: u8, file: u8) -> Self {
		if rank > 32 {
			panic!("A Square cannot have a rank greater than 32. Got {}", rank)
		} else if file > 32 {
			panic!("A Square cannot have a file greater than 32. Got {}", file)
		} else {
			Self { rank, file }
		}
	}

	pub fn from_value(value: usize) -> Self {
		static VALUE_COORDINATE_MAP: [SquareCoordinate; 32] = [
			SquareCoordinate { rank: 0, file: 6 },
			SquareCoordinate { rank: 1, file: 7 },
			SquareCoordinate { rank: 4, file: 0 },
			SquareCoordinate { rank: 5, file: 1 },
			SquareCoordinate { rank: 6, file: 2 },
			SquareCoordinate { rank: 7, file: 3 },
			SquareCoordinate { rank: 0, file: 4 },
			SquareCoordinate { rank: 1, file: 5 },
			SquareCoordinate { rank: 2, file: 6 },
			SquareCoordinate { rank: 3, file: 7 },
			SquareCoordinate { rank: 6, file: 0 },
			SquareCoordinate { rank: 7, file: 1 },
			SquareCoordinate { rank: 0, file: 2 },
			SquareCoordinate { rank: 1, file: 3 },
			SquareCoordinate { rank: 2, file: 4 },
			SquareCoordinate { rank: 3, file: 5 },
			SquareCoordinate { rank: 4, file: 6 },
			SquareCoordinate { rank: 5, file: 7 },
			SquareCoordinate { rank: 0, file: 0 },
			SquareCoordinate { rank: 1, file: 1 },
			SquareCoordinate { rank: 2, file: 2 },
			SquareCoordinate { rank: 3, file: 3 },
			SquareCoordinate { rank: 4, file: 4 },
			SquareCoordinate { rank: 5, file: 5 },
			SquareCoordinate { rank: 6, file: 6 },
			SquareCoordinate { rank: 7, file: 7 },
			SquareCoordinate { rank: 2, file: 0 },
			SquareCoordinate { rank: 3, file: 1 },
			SquareCoordinate { rank: 4, file: 2 },
			SquareCoordinate { rank: 5, file: 3 },
			SquareCoordinate { rank: 6, file: 4 },
			SquareCoordinate { rank: 7, file: 5 },
		];

		VALUE_COORDINATE_MAP[value]
	}

	pub fn rank(self) -> u8 {
		self.rank
	}

	pub fn file(self) -> u8 {
		self.file
	}

	pub fn to_value(self) -> Option<usize> {
		if self.rank % 2 == 0 {
			if self.file % 2 == 0 {
				Some(((18 - ((self.file / 2) * 6)) + ((self.rank / 2) * 8)) as usize % 32)
			} else {
				None
			}
		} else if self.file % 2 == 1 {
			let column_value = match self.file {
				1 => 19,
				3 => 13,
				5 => 7,
				7 => 1,
				_ => unreachable!(),
			};
			let row_value = match self.rank {
				1 => 0,
				3 => 8,
				5 => 16,
				7 => 24,
				_ => unreachable!(),
			};
			Some((column_value + row_value) % 32)
		} else {
			None
		}
	}
}

impl Display for SquareCoordinate {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}{}",
			char::from_u32((self.file + b'a') as u32).unwrap(),
			self.rank + 1
		)
	}
}
