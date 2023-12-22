#![feature(new_uninit)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]

use std::num::{NonZeroU8, NonZeroUsize};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use eval::{evaluate, Evaluation};

pub use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves};
pub use transposition_table::{TranspositionTable, TranspositionTableRef};

mod eval;
mod lazysort;
mod stackvec;
mod tablebase;
mod transposition_table;

pub const ENGINE_NAME: &str = "Ampere";
pub const ENGINE_AUTHOR: &str = "Mica White";
pub const ENGINE_ABOUT: &str = "Ampere Checkers Bot v1.0\nCopyright Mica White";

pub struct Engine<'a> {
	position: Mutex<CheckersBitBoard>,
	transposition_table: TranspositionTable,

	debug: AtomicBool,
	frontend: &'a dyn Frontend,

	current_thread: Mutex<Option<JoinHandle<Evaluation>>>,
	current_task: Mutex<Option<Arc<EvaluationTask<'a>>>>,
	pondering_task: Mutex<Option<Arc<EvaluationTask<'a>>>>,
}

struct EvaluationTask<'a> {
	position: CheckersBitBoard,
	transposition_table: TranspositionTableRef<'a>,
	allowed_moves: Option<Arc<[Move]>>,
	limits: ActualLimit,
	ponder: bool,
	cancel_flag: AtomicBool,
	end_ponder_flag: AtomicBool,

	start_time: Instant,
	current_depth: AtomicU8,
	selective_depth: AtomicU8,
	nodes_explored: AtomicUsize,
	principle_variation: Mutex<Vec<Move>>,
}

#[derive(Debug, Default, Clone)]
pub struct EvaluationSettings {
	pub restrict_moves: Option<Arc<[Move]>>,
	pub ponder: bool,
	pub clock: Clock,
	pub search_until: SearchLimit,
}

impl EvaluationSettings {
	fn get_limits(&self, this_color: PieceColor) -> ActualLimit {
		match &self.search_until {
			SearchLimit::Infinite => ActualLimit::default(),
			SearchLimit::Limited(limit) => *limit,
			SearchLimit::Auto => ActualLimit {
				nodes: None,
				depth: NonZeroU8::new(30),
				time: Some(self.clock.recommended_time(this_color)),
			},
		}
	}
}

#[derive(Debug, Clone)]
pub enum Clock {
	Unlimited,
	TimePerMove(Duration),
	ChessClock {
		white_time_remaining: Duration,
		black_time_remaining: Duration,
		white_increment: Duration,
		black_increment: Duration,
		moves_until_next_time_control: Option<(u32, Duration)>,
	},
}

impl Clock {
	fn recommended_time(&self, this_color: PieceColor) -> Duration {
		match self {
			Self::Unlimited => Duration::from_secs(60 * 5), // 5 minutes
			Self::TimePerMove(time) => *time,
			Self::ChessClock {
				white_time_remaining,
				black_time_remaining,
				white_increment,
				black_increment,
				moves_until_next_time_control,
			} => {
				let my_time = match this_color {
					PieceColor::Dark => black_time_remaining,
					PieceColor::Light => white_time_remaining,
				};
				let my_increment = match this_color {
					PieceColor::Dark => black_increment,
					PieceColor::Light => white_increment,
				};

				// TODO this could certainly be better
				let moves_to_go = moves_until_next_time_control.map(|m| m.0).unwrap_or(50);

				(my_time.checked_div(moves_to_go).unwrap_or(*my_time) + *my_increment).div_f32(1.25)
			}
		}
	}
}

impl Default for Clock {
	fn default() -> Self {
		Self::TimePerMove(Duration::from_secs(60 * 5))
	}
}

#[derive(Debug, Default, Clone)]
pub enum SearchLimit {
	#[default]
	Auto,
	Infinite,
	Limited(ActualLimit),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ActualLimit {
	pub nodes: Option<NonZeroUsize>,
	pub depth: Option<NonZeroU8>,
	pub time: Option<Duration>,
}

pub trait Frontend: Sync {
	fn debug(&self, msg: &str);

	fn report_best_move(&self, best_move: Move);
}

impl<'a> Engine<'a> {
	pub fn new(transposition_table_size: usize, frontend: &'a dyn Frontend) -> Self {
		Self {
			position: Mutex::new(CheckersBitBoard::starting_position()),
			transposition_table: TranspositionTable::new(transposition_table_size),

			debug: AtomicBool::new(false),
			frontend,

			current_thread: Mutex::new(None),
			current_task: Mutex::new(None),
			pondering_task: Mutex::new(None),
		}
	}

	pub fn set_debug(&self, debug: bool) {
		self.debug.store(debug, Ordering::Release);
	}

	pub fn is_legal_move(&self, checker_move: Move) -> bool {
		let position = self.position.lock();
		PossibleMoves::moves(*position).contains(checker_move)
	}

	pub fn reset_position(&self) {
		self.set_position(CheckersBitBoard::starting_position())
	}

	pub fn set_position(&self, position: CheckersBitBoard) {
		let mut position_ptr = self.position.lock();
		*position_ptr = position;
	}

	pub fn apply_move(&self, checker_move: Move) -> Option<()> {
		unsafe {
			if self.is_legal_move(checker_move) {
				let mut position = self.position.lock();
				*position = checker_move.apply_to(*position);
				Some(())
			} else {
				None
			}
		}
	}

	pub fn start_evaluation(&'static self, settings: EvaluationSettings) {
		// finish the pondering thread
		let mut pondering_task = self.pondering_task.lock();
		if let Some(task) = pondering_task.take() {
			task.end_ponder_flag.store(true, Ordering::Release);
		}

		let position = *self.position.lock();
		let transposition_table = self.transposition_table.get_ref();
		let limits = settings.get_limits(position.turn());
		let allowed_moves = settings.restrict_moves;
		let ponder = settings.ponder;
		let cancel_flag = AtomicBool::new(false);
		let end_ponder_flag = AtomicBool::new(false);

		let start_time = Instant::now();
		let current_depth = AtomicU8::new(0);
		let selective_depth = AtomicU8::new(0);
		let nodes_explored = AtomicUsize::new(0);
		let principle_variation = Mutex::new(Vec::new());

		let task = EvaluationTask {
			position,
			transposition_table,
			allowed_moves,
			limits,
			ponder,
			cancel_flag,
			end_ponder_flag,

			start_time,
			current_depth,
			selective_depth,
			nodes_explored,
			principle_variation,
		};

		let task = Arc::new(task);
		let task_ref = task.clone();
		let mut task_ptr = self.current_task.lock();
		*task_ptr = Some(task);

		if ponder {
			let mut pondering_task = self.pondering_task.lock();
			*pondering_task = Some(task_ref.clone());
		}

		let thread = std::thread::spawn(move || evaluate(task_ref, self.frontend));
		let mut thread_ptr = self.current_thread.lock();
		*thread_ptr = Some(thread);
	}

	pub fn stop_evaluation(&self) -> Option<()> {
		let current_task = self.current_task.lock().take()?;
		current_task.cancel_flag.store(true, Ordering::Release);

		self.current_thread.lock().take();

		Some(())
	}
}
