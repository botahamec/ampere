use ai::CheckersBitBoard;
pub fn eval() -> f32 {
	ai::eval(12, CheckersBitBoard::starting_position())
}
