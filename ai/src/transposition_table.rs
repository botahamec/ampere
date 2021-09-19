use crate::CheckersBitBoard;
use parking_lot::lock_api::RawMutex;
use parking_lot::Mutex;

#[cfg(debug_assertions)]
const TABLE_SIZE: usize = 1_000_000 / std::mem::size_of::<TranspositionTableEntry>();

#[cfg(not(debug_assertions))]
const TABLE_SIZE: usize = 10_000_000 / std::mem::size_of::<TranspositionTableEntry>();

const EMPTY_ENTRY: Option<TranspositionTableEntry> = None;
static mut REPLACE_TABLE: [Option<TranspositionTableEntry>; TABLE_SIZE] = [EMPTY_ENTRY; TABLE_SIZE];
static mut DEPTH_TABLE: [Option<TranspositionTableEntry>; TABLE_SIZE] = [EMPTY_ENTRY; TABLE_SIZE];

#[derive(Copy, Clone, Debug)]
struct TranspositionTableEntry {
	board: CheckersBitBoard,
	eval: f32,
	depth: u8,
}

pub struct TranspositionTableReference {
	replace_table: &'static mut [Option<TranspositionTableEntry>; TABLE_SIZE],
	depth_table: &'static mut [Option<TranspositionTableEntry>; TABLE_SIZE],
}

impl TranspositionTableEntry {
	const fn new(board: CheckersBitBoard, eval: f32, depth: u8) -> Self {
		Self { board, eval, depth }
	}
}

impl TranspositionTableReference {
	pub fn new() -> Self {
		Self {
			replace_table: unsafe { &mut REPLACE_TABLE },
			depth_table: unsafe { &mut DEPTH_TABLE },
		}
	}

	pub fn get(self, board: CheckersBitBoard, depth: u8) -> Option<f32> {
		// try the replace table
		let entry = unsafe {
			self.replace_table
				.get_unchecked(board.hash_code() as usize % TABLE_SIZE)
		};
		if let Some(entry) = *entry {
			if entry.board == board && entry.depth >= depth {
				return Some(entry.eval);
			}
		}

		// try the depth table
		let entry = unsafe {
			self.depth_table
				.get_unchecked(board.hash_code() as usize % TABLE_SIZE)
		};
		match *entry {
			Some(entry) => {
				if entry.board == board {
					if entry.depth >= depth {
						Some(entry.eval)
					} else {
						None
					}
				} else {
					None
				}
			}
			None => None,
		}
	}

	pub fn insert(self, board: CheckersBitBoard, eval: f32, depth: u8) {
		// insert to the replace table
		let entry = unsafe {
			self.replace_table
				.get_unchecked_mut(board.hash_code() as usize % TABLE_SIZE)
		};
		*entry = Some(TranspositionTableEntry::new(board, eval, depth));

		// insert to the depth table, only if the new depth is higher
		let entry = unsafe {
			self.depth_table
				.get_unchecked_mut(board.hash_code() as usize % TABLE_SIZE)
		};
		match *entry {
			Some(entry_val) => {
				if depth >= entry_val.depth {
					*entry = Some(TranspositionTableEntry::new(board, eval, depth));
				}
			}
			None => *entry = Some(TranspositionTableEntry::new(board, eval, depth)),
		}
	}
}
