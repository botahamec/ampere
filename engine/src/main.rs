use engine::{current_evaluation, CheckersBitBoard, TranspositionTable};
use mimalloc::MiMalloc;

#[global_allocator]
static ALLOCATOR: MiMalloc = MiMalloc;

const DEPTH: u8 = 18;

fn main() {
	let board = CheckersBitBoard::starting_position();
	let mut table = TranspositionTable::new(50_000);
	println!("{}", current_evaluation(DEPTH, board, table.mut_ref()));
}
