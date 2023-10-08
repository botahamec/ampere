use crate::{eval::Evaluation, CheckersBitBoard};
use parking_lot::RwLock;
use std::num::NonZeroU8;

#[derive(Copy, Clone, Debug)]
struct TranspositionTableEntry {
	board: CheckersBitBoard,
	eval: Evaluation,
	depth: NonZeroU8,
}

impl TranspositionTableEntry {
	const fn new(board: CheckersBitBoard, eval: Evaluation, depth: NonZeroU8) -> Self {
		Self { board, eval, depth }
	}
}

pub struct TranspositionTable {
	replace_table: Box<[RwLock<Option<TranspositionTableEntry>>]>,
	depth_table: Box<[RwLock<Option<TranspositionTableEntry>>]>,
}

#[derive(Copy, Clone, Debug)]
pub struct TranspositionTableRef<'a> {
	replace_table: &'a [RwLock<Option<TranspositionTableEntry>>],
	depth_table: &'a [RwLock<Option<TranspositionTableEntry>>],
}

impl<'a> TranspositionTableRef<'a> {
	pub fn get(self, board: CheckersBitBoard, depth: u8) -> Option<Evaluation> {
		let table_len = self.replace_table.as_ref().len();

		// try the replace table
		let entry = unsafe {
			self.replace_table
				.as_ref()
				.get_unchecked(board.hash_code() as usize % table_len)
				.read()
		};
		if let Some(entry) = *entry {
			if entry.board == board && entry.depth.get() >= depth {
				return Some(entry.eval);
			}
		}

		// try the depth table
		let entry = unsafe {
			self.depth_table
				.as_ref()
				.get_unchecked(board.hash_code() as usize % table_len)
				.read()
		};
		match *entry {
			Some(entry) => {
				if entry.board == board {
					if entry.depth.get() >= depth {
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

	pub fn get_any_depth(self, board: CheckersBitBoard) -> Option<Evaluation> {
		let table_len = self.replace_table.as_ref().len();

		// try the depth table
		let entry = unsafe {
			self.depth_table
				.as_ref()
				.get_unchecked(board.hash_code() as usize % table_len)
				.read()
		};
		if let Some(entry) = *entry {
			if entry.board == board {
				return Some(entry.eval);
			}
		}

		// try the replace table
		let entry = unsafe {
			self.replace_table
				.as_ref()
				.get_unchecked(board.hash_code() as usize % table_len)
				.read()
		};
		match *entry {
			Some(entry) => {
				if entry.board == board {
					Some(entry.eval)
				} else {
					None
				}
			}
			None => None,
		}
	}

	pub fn insert(&self, board: CheckersBitBoard, eval: Evaluation, depth: NonZeroU8) {
		let table_len = self.replace_table.as_ref().len();

		// insert to the replace table
		let mut entry = unsafe {
			self.replace_table
				.get_unchecked(board.hash_code() as usize % table_len)
				.write()
		};
		*entry = Some(TranspositionTableEntry::new(board, eval, depth));

		// insert to the depth table, only if the new depth is higher
		let mut entry = unsafe {
			self.depth_table
				.get_unchecked(board.hash_code() as usize % table_len)
				.write()
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

impl TranspositionTable {
	pub fn new(table_size: usize) -> Self {
		let mut replace_table = Box::new_uninit_slice(table_size / 2);
		let mut depth_table = Box::new_uninit_slice(table_size / 2);

		for entry in replace_table.iter_mut() {
			entry.write(RwLock::new(None));
		}

		for entry in depth_table.iter_mut() {
			entry.write(RwLock::new(None));
		}

		Self {
			replace_table: unsafe { replace_table.assume_init() },
			depth_table: unsafe { depth_table.assume_init() },
		}
	}

	pub fn mut_ref(&mut self) -> TranspositionTableRef {
		TranspositionTableRef {
			replace_table: &self.replace_table,
			depth_table: &self.depth_table,
		}
	}
}
