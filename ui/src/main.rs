use engine::TranspositionTable;
use model::{CheckersBitBoard, Move, PieceColor, PossibleMoves, SquareCoordinate};
use std::collections::HashSet;
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::input::MouseButton;
use tetra::math::Vec2;
use tetra::{input, Context, ContextBuilder, State};

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;
const DARK_SLATE_BLUE: Color = Color::rgb(0.2823529, 0.2392157, 0.545098);

struct GameState {
	chess_board: Texture,
	possible_move_square: Texture,
	dark_piece: Texture,
	light_piece: Texture,
	dark_king: Texture,
	light_king: Texture,
	bit_board: CheckersBitBoard,
	selected_square: Option<SquareCoordinate>,
	possible_moves: HashSet<Move>,
	evaluation: f32,
	transposition_table: TranspositionTable,
}

impl GameState {
	fn new(ctx: &mut Context) -> tetra::Result<Self> {
		Ok(GameState {
			chess_board: Texture::new(ctx, "./ui/resources/chess_board.png")?,
			possible_move_square: Texture::new(ctx, "./ui/resources/possible_move.png")?,
			dark_piece: Texture::new(ctx, "./ui/resources/red_piece.png")?,
			light_piece: Texture::new(ctx, "./ui/resources/white_piece.png")?,
			dark_king: Texture::new(ctx, "./ui/resources/red_king.png")?,
			light_king: Texture::new(ctx, "./ui/resources/white_king.png")?,
			bit_board: CheckersBitBoard::starting_position(),
			selected_square: None,
			possible_moves: HashSet::new(),
			evaluation: 0.0,
			transposition_table: TranspositionTable::new(5_000_000 / 18),
		})
	}
}

impl GameState {
	fn draw_highlighted_square(&self, ctx: &mut Context, square: SquareCoordinate) {
		let square_draw_params = DrawParams::new()
			.position(Vec2::new(
				120.0 + (50.0 * square.file() as f32),
				390.0 - (50.0 * square.rank() as f32),
			))
			.scale(Vec2::new(0.5, 0.5));

		self.possible_move_square.draw(ctx, square_draw_params);
	}
}

impl State for GameState {
	fn update(&mut self, ctx: &mut Context) -> tetra::Result {
		if input::is_mouse_button_released(ctx, MouseButton::Left) {
			let x = input::get_mouse_x(ctx);
			let y = input::get_mouse_y(ctx);

			if x > 120.0 && y > 40.0 && x < 520.0 && y < 440.0 {
				let file = ((x - 140.0) / 50.0).round();
				let rank = ((410.0 - y) / 50.0).round();
				let square = SquareCoordinate::new(rank as u8, file as u8);

				if self
					.possible_moves
					.iter()
					.map(|m| SquareCoordinate::from_value(m.end_position()))
					.collect::<HashSet<SquareCoordinate>>()
					.contains(&square)
				{
					let selected_move = self
						.possible_moves
						.iter()
						.find(|m| SquareCoordinate::from_value(m.end_position()) == square)
						.unwrap();

					// safety: this was determined to be in the list of possible moves
					self.bit_board = unsafe { selected_move.apply_to(self.bit_board) };

					let evaluation = engine::current_evaluation(
						7,
						self.bit_board,
						self.transposition_table.mut_ref(),
					);
					println!("AI advantage: {}", evaluation);

					// ai makes a move
					while self.bit_board.turn() == PieceColor::Light
						&& !PossibleMoves::moves(self.bit_board).is_empty()
					{
						let best_move = dbg!(engine::best_move(
							14,
							self.bit_board,
							self.transposition_table.mut_ref()
						));
						self.bit_board = unsafe { best_move.apply_to(self.bit_board) };
					}

					self.selected_square = None;
					self.possible_moves.clear();
					let evaluation = engine::current_evaluation(
						7,
						self.bit_board,
						self.transposition_table.mut_ref(),
					);
					println!("Your advantage: {}", evaluation);
				} else {
					self.selected_square = Some(square);

					let moves = PossibleMoves::moves(self.bit_board);
					self.possible_moves = moves
						.into_iter()
						.filter(|m| SquareCoordinate::from_value(m.start() as usize) == square)
						.collect();
				}
			}
		}

		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
		graphics::clear(ctx, DARK_SLATE_BLUE);

		let board_draw_params = DrawParams::new()
			.position(Vec2::new(120.0, 40.0))
			.scale(Vec2::new(0.4938272, 0.4938272));

		self.chess_board.draw(ctx, board_draw_params);

		if let Some(square) = self.selected_square {
			self.draw_highlighted_square(ctx, square);
		}

		for possible_move in &self.possible_moves {
			self.draw_highlighted_square(
				ctx,
				SquareCoordinate::from_value(possible_move.end_position()),
			);
		}

		for row in 0..8 {
			for col in 0..8 {
				if let Some(piece) = self.bit_board.get_at_row_col(row, col) {
					let piece_draw_params = DrawParams::new()
						.position(Vec2::new(
							130.0 + (50.0 * col as f32),
							400.0 - (50.0 * row as f32),
						))
						.scale(Vec2::new(0.3, 0.3));

					match piece.color() {
						PieceColor::Dark => {
							if piece.is_king() {
								self.dark_king.draw(ctx, piece_draw_params)
							} else {
								self.dark_piece.draw(ctx, piece_draw_params)
							}
						}
						PieceColor::Light => {
							if piece.is_king() {
								self.light_king.draw(ctx, piece_draw_params)
							} else {
								self.light_piece.draw(ctx, piece_draw_params)
							}
						}
					}
				}
			}
		}

		Ok(())
	}
}

fn main() -> tetra::Result {
	let title = "Checkers with Ampere";
	let mut builder = ContextBuilder::new(title, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);

	builder.show_mouse(true);
	builder.quit_on_escape(true);

	if cfg!(debug_assertions) {
		builder.debug_info(true);
	} else {
		builder.debug_info(false);
	}

	builder.build()?.run(GameState::new)
}
