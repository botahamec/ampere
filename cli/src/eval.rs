use ai::{CheckersBitBoard, Move};
pub fn eval(depth: usize) -> f32 {
	ai::eval(depth, 0.0, 1.0, CheckersBitBoard::starting_position())
}

pub fn best_move(depth: usize) -> Move {
	ai::best_move(depth, CheckersBitBoard::starting_position())
}
