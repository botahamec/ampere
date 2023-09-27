use engine::{negamax, CheckersBitBoard, TranspositionTable};

const DEPTH: u8 = 18;

fn main() {
	let board = CheckersBitBoard::starting_position();
	let mut table = TranspositionTable::new(50_000);
	let mut alpha = -1.0;
	let mut beta = 1.0;
	for i in 0..DEPTH {
		let mut eval = negamax(i, alpha, beta, board, table.mut_ref());

		if (eval <= alpha) || (eval >= beta) {
			eval = negamax(i, -1.0, 1.0, board, table.mut_ref());
		}

		alpha = f32::max(eval + 0.125, -1.0);
		beta = f32::min(eval + 0.125, 1.0);
	}

	println!("{:?}", negamax(DEPTH, alpha, beta, board, table.mut_ref(),));
}
