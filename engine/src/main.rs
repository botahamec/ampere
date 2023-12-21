use std::{num::NonZeroU8, thread::sleep, time::Duration};

use engine::{ActualLimit, Engine, EvaluationSettings, Frontend};
use mimalloc::MiMalloc;

#[global_allocator]
static ALLOCATOR: MiMalloc = MiMalloc;

const DEPTH: u8 = 19;

struct BasicFrontend;

impl Frontend for BasicFrontend {
	fn debug(&self, msg: &str) {}

	fn report_best_move(&self, best_move: model::Move) {
		println!("{best_move}");
		std::process::exit(0);
	}
}

fn main() {
	let engine = Box::leak(Box::new(Engine::new(1_000_000, &BasicFrontend)));
	engine.start_evaluation(EvaluationSettings {
		restrict_moves: None,
		ponder: false,
		clock: engine::Clock::Unlimited,
		search_until: engine::SearchLimit::Limited(ActualLimit {
			nodes: None,
			depth: Some(NonZeroU8::new(DEPTH).unwrap()),
			time: None,
		}),
	});

	loop {
		sleep(Duration::from_millis(200))
	}
}
