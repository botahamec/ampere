use model::{CheckersBitBoard, PieceColor};
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;
const DARK_SLATE_BLUE: Color = Color::rgb(0.2823529, 0.2392157, 0.5450980);

struct GameState {
	chess_board: Texture,
	dark_piece: Texture,
	light_piece: Texture,
	dark_king: Texture,
	light_king: Texture,
	bit_board: CheckersBitBoard,
}

impl GameState {
	fn new(ctx: &mut Context) -> tetra::Result<Self> {
		Ok(GameState {
			chess_board: Texture::new(ctx, "./ui/resources/chess_board.png")?,
			dark_piece: Texture::new(ctx, "./ui/resources/red_piece.png")?,
			light_piece: Texture::new(ctx, "./ui/resources/white_piece.png")?,
			dark_king: Texture::new(ctx, "./ui/resources/red_king.png")?,
			light_king: Texture::new(ctx, "./ui/resources/white_king.png")?,
			bit_board: CheckersBitBoard::starting_position(),
		})
	}
}

impl State for GameState {
	fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
		graphics::clear(ctx, DARK_SLATE_BLUE);

		let board_draw_params = DrawParams::new()
			.position(Vec2::new(120.0, 40.0))
			.scale(Vec2::new(0.4938272, 0.4938272));

		self.chess_board.draw(ctx, board_draw_params);

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
