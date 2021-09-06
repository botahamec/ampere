use ai::CheckersBitBoard;
pub fn eval(depth: usize) -> f32 {
	ai::eval(depth, 0.0, 1.0, CheckersBitBoard::starting_position())
}
