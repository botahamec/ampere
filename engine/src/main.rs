use std::num::NonZeroU8;

use engine::{ActualLimit, Engine, EvaluationSettings, Frontend};
use mimalloc::MiMalloc;
use model::CheckersBitBoard;

#[global_allocator]
static ALLOCATOR: MiMalloc = MiMalloc;

const DEPTH: u8 = 19;

struct BasicFrontend;

impl Frontend for BasicFrontend {
	fn debug(&self, msg: &str) {
		println!("{msg}");
	}

	fn report_best_move(&self, best_move: model::Move) {
		println!("{best_move}");
	}
}

fn main() {
	let engine = Box::leak(Box::new(Engine::new(1_000_000, &BasicFrontend)));
	let (_, best) = engine.evaluate(
		None,
		EvaluationSettings {
			restrict_moves: None,
			ponder: false,
			clock: engine::Clock::Unlimited,
			search_until: engine::SearchLimit::Limited(ActualLimit {
				nodes: None,
				depth: Some(NonZeroU8::new(DEPTH).unwrap()),
				time: None,
			}),
		},
	);
	engine.set_position(CheckersBitBoard::new(
		4294967295,
		2206409603,
		3005432691,
		model::PieceColor::Light,
	));
	engine.evaluate(
		None,
		EvaluationSettings {
			restrict_moves: None,
			ponder: false,
			clock: engine::Clock::Unlimited,
			search_until: engine::SearchLimit::Limited(ActualLimit {
				nodes: None,
				depth: Some(NonZeroU8::new(DEPTH).unwrap()),
				time: None,
			}),
		},
	);
}
