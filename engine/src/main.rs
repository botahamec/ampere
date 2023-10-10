use engine::{current_evaluation, CheckersBitBoard, TranspositionTable};

const DEPTH: u8 = 18;

fn main() {
	let board = CheckersBitBoard::starting_position();
	let mut table = TranspositionTable::new(50_000);
	println!("{}", current_evaluation(DEPTH, board, table.mut_ref()));
}
