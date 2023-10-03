use std::{iter::Peekable, sync::Arc};

use crate::tokens::{Color, PdnToken, PdnTokenBody, TokenHeader};

#[derive(Debug, Clone)]
pub struct PdnFile {
	games: Vec<Game>,
	game_separators: Vec<TokenHeader>,
}

#[derive(Debug, Clone)]
pub struct Game {
	header: Vec<PdnTag>,
	body: Vec<BodyPart>,
}

#[derive(Debug, Clone)]
pub struct PdnTag {
	left_bracket: TokenHeader,
	identifier_token: TokenHeader,
	string_token: TokenHeader,
	right_bracket: TokenHeader,

	identifier: Arc<str>,
	string: Arc<str>,
}

#[derive(Debug, Clone)]
pub enum BodyPart {
	Move(GameMove),
	Variation(Variation),
	Comment(TokenHeader, Arc<str>),
	Setup(TokenHeader, Arc<str>),
	Nag(TokenHeader, usize),
}

#[derive(Debug, Clone)]
pub struct Variation {
	left_parenthesis: TokenHeader,
	body: Vec<BodyPart>,
	right_parenthesis: TokenHeader,
}

#[derive(Debug, Clone)]
pub struct GameMove {
	move_number: Option<(TokenHeader, usize, Color)>,
	game_move: Move,
	move_strength: Option<(TokenHeader, Arc<str>)>,
}

#[derive(Debug, Clone)]
pub enum Move {
	Normal(Square, TokenHeader, Square),
	Capture(Square, Vec<(TokenHeader, Square)>),
}

#[derive(Debug, Clone)]
pub enum Square {
	Alpha(TokenHeader, char, char),
	Num(TokenHeader, u8),
}

/// Returns `Ok` if parsed successfully. If there are no tokens left,
/// `Err(None)` is returned. If the next token is not a square position, then
/// `Err(Some(token))` is returned.
fn parse_square(scanner: &mut impl Iterator<Item = PdnToken>) -> Result<Square, Option<PdnToken>> {
	let Some(token) = scanner.next() else {
		return Err(None);
	};
	let header = token.header;
	let body = &token.body;

	match *body {
		PdnTokenBody::AlphaSquare(letter, number) => Ok(Square::Alpha(header, letter, number)),
		PdnTokenBody::NumSquare(number) => Ok(Square::Num(header, number)),
		_ => Err(Some(token)),
	}
}

#[derive(Debug, Clone)]
pub enum MoveError {
	EndOfFile,
	NoStartSquare(Option<PdnToken>),
	NoEndSquare(Option<PdnToken>),
	InvalidCaptureSquares(Vec<Option<PdnToken>>),
	NoMoveSeparator,
}

fn parse_normal_move(
	first_square: Square,
	scanner: &mut impl Iterator<Item = PdnToken>,
) -> Result<Move, MoveError> {
	let Some(separator) = scanner.next() else {
		return Err(MoveError::NoMoveSeparator);
	};
	let square = match parse_square(scanner) {
		Ok(square) => square,
		Err(error) => return Err(MoveError::NoEndSquare(error)),
	};
	Ok(Move::Normal(first_square, separator.header, square))
}

fn parse_capture_move(
	first_square: Square,
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
) -> Result<Move, MoveError> {
	let mut captures = Vec::new();
	let mut errors = Vec::new();

	while let Some(token) = scanner.peek() {
		if token.body != PdnTokenBody::CaptureSeparator {
			break;
		}

		let separator = scanner.next().expect("separator should be next");
		match parse_square(scanner) {
			Ok(square) => captures.push((separator.header, square)),
			Err(error) => errors.push(error),
		}
	}

	if !errors.is_empty() {
		Err(MoveError::InvalidCaptureSquares(errors))
	} else {
		Ok(Move::Capture(first_square, captures))
	}
}

fn parse_move(scanner: &mut Peekable<impl Iterator<Item = PdnToken>>) -> Result<Move, MoveError> {
	let square = match parse_square(scanner) {
		Ok(square) => square,
		Err(error) => return Err(MoveError::NoStartSquare(error)),
	};

	let Some(token) = scanner.peek() else {
		return Err(MoveError::NoMoveSeparator);
	};
	let body = &token.body;

	match body {
		PdnTokenBody::MoveSeparator => parse_normal_move(square, scanner),
		PdnTokenBody::CaptureSeparator => parse_capture_move(square, scanner),
		_ => Err(MoveError::NoMoveSeparator),
	}
}

#[derive(Debug, Clone)]
pub enum GameMoveError {
	EndOfFile,
	BadMove(MoveError),
}

fn whitespace_if_found(
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
) -> Option<TokenHeader> {
	let token = scanner.peek()?;
	if let PdnTokenBody::Space(_) = token.body {
		Some(scanner.next()?.header)
	} else {
		None
	}
}

fn parse_game_move(
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
) -> Result<GameMove, GameMoveError> {
	let Some(next_token) = scanner.peek() else {
		return Err(GameMoveError::EndOfFile);
	};

	let move_number = match next_token.body {
		PdnTokenBody::MoveNumber(number, color) => Some((next_token.header, number, color)),
		_ => None,
	};

	if move_number.is_some() {
		scanner.next();
	}

	whitespace_if_found(scanner);

	let game_move = parse_move(scanner);

	let move_strength = if let Some(token) = scanner.peek() {
		if let PdnTokenBody::MoveStrength(string) = &token.body {
			Some((token.header, string.clone()))
		} else {
			None
		}
	} else {
		None
	};

	if move_strength.is_some() {
		scanner.next();
	}

	match game_move {
		Ok(game_move) => Ok(GameMove {
			move_number,
			game_move,
			move_strength,
		}),
		Err(error) => Err(GameMoveError::BadMove(error)),
	}
}

#[derive(Debug, Clone)]
pub enum VariationError {
	UnexpectedEnd(BodyError),
	BadBody(BodyError),
}

fn parse_variation(
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
) -> Result<Variation, VariationError> {
	let left_parenthesis = scanner.next().expect("should start with left paren").header;
	let body = parse_body_until(scanner, PdnTokenBody::RightParenthesis)?;
	let right_parenthesis = scanner.next().expect("should end with right paren").header;

	Ok(Variation {
		left_parenthesis,
		body,
		right_parenthesis,
	})
}

#[derive(Debug, Clone)]
pub enum BodyPartError {
	EndOfFile,
	InvalidToken(PdnToken),
	BadMove(GameMoveError),
	BadVariation(VariationError),
}

fn parse_body_part(
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
) -> Result<BodyPart, BodyPartError> {
	let Some(token) = scanner.peek() else {
		return Err(BodyPartError::EndOfFile);
	};

	match &token.body {
		PdnTokenBody::MoveNumber(..)
		| PdnTokenBody::AlphaSquare(..)
		| PdnTokenBody::NumSquare(..) => match parse_game_move(scanner) {
			Ok(mov) => Ok(BodyPart::Move(mov)),
			Err(error) => Err(BodyPartError::BadMove(error)),
		},
		PdnTokenBody::LeftParenthesis => match parse_variation(scanner) {
			Ok(variation) => Ok(BodyPart::Variation(variation)),
			Err(error) => Err(BodyPartError::BadVariation(error)),
		},
		PdnTokenBody::Comment(string) => Ok(BodyPart::Comment(token.header, string.clone())),
		PdnTokenBody::Setup(string) => Ok(BodyPart::Setup(token.header, string.clone())),
		PdnTokenBody::Nag(number) => Ok(BodyPart::Nag(token.header, *number)),
		_ => Err(BodyPartError::InvalidToken(token.clone())),
	}
}

pub type BodyError = Vec<Result<BodyPart, BodyPartError>>;

fn parse_body_until(
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
	until: PdnTokenBody,
) -> Result<Vec<BodyPart>, VariationError> {
	let mut parts = Vec::new();

	loop {
		whitespace_if_found(scanner);

		let Some(token) = scanner.peek() else {
			return Err(VariationError::UnexpectedEnd(parts));
		};

		if token.body == until {
			break;
		}

		parts.push(parse_body_part(scanner));
		whitespace_if_found(scanner);
	}

	if parts.iter().any(|r| r.is_err()) {
		Err(VariationError::BadBody(parts))
	} else {
		Ok(parts.iter().map(|r| r.as_ref().cloned().unwrap()).collect())
	}
}

#[derive(Debug, Clone)]
pub enum PdnTagError {
	EndOfFile,
	NoStartBracket(PdnToken),
	Unterminated(Vec<PdnToken>),
	NoIdentifier,
	NoString,
	NoEndBracket,
}

fn parse_pdn_tag(
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
) -> Result<PdnTag, PdnTagError> {
	whitespace_if_found(scanner);

	let Some(left_bracket) = scanner.next() else {
		return Err(PdnTagError::EndOfFile);
	};

	if left_bracket.body != PdnTokenBody::LeftBracket {
		return Err(PdnTagError::NoStartBracket(left_bracket));
	}

	whitespace_if_found(scanner);

	let Some(identifier_token) = scanner.next() else {
		return Err(PdnTagError::Unterminated(vec![left_bracket]));
	};

	let PdnTokenBody::Identifier(identifier) = &identifier_token.body else {
		return Err(PdnTagError::NoIdentifier);
	};

	whitespace_if_found(scanner);

	let Some(value_token) = scanner.next() else {
		return Err(PdnTagError::Unterminated(vec![
			left_bracket,
			identifier_token,
		]));
	};

	let PdnTokenBody::String(value) = &value_token.body else {
		return Err(PdnTagError::NoIdentifier);
	};

	whitespace_if_found(scanner);

	let Some(right_bracket) = scanner.next() else {
		return Err(PdnTagError::Unterminated(vec![
			left_bracket,
			identifier_token,
			value_token,
		]));
	};

	if right_bracket.body != PdnTokenBody::RightBracket {
		return Err(PdnTagError::NoEndBracket);
	}

	whitespace_if_found(scanner);

	Ok(PdnTag {
		left_bracket: left_bracket.header,
		identifier_token: identifier_token.header,
		string_token: value_token.header,
		right_bracket: right_bracket.header,
		identifier: identifier.clone(),
		string: value.clone(),
	})
}

pub type HeaderError = Vec<Result<PdnTag, PdnTagError>>;

fn parse_header(
	scanner: &mut Peekable<impl Iterator<Item = PdnToken>>,
) -> Result<Vec<PdnTag>, HeaderError> {
	let mut tags = Vec::new();

	loop {
		let Some(token) = scanner.peek() else {
			break;
		};

		if token.body != PdnTokenBody::LeftBracket {
			break;
		}

		tags.push(parse_pdn_tag(scanner));
	}

	if tags.iter().any(|r| r.is_err()) {
		Err(tags)
	} else {
		Ok(tags.iter().map(|r| r.as_ref().cloned().unwrap()).collect())
	}
}

#[derive(Debug, Clone)]
pub struct GameError {
	header: Result<Vec<PdnTag>, HeaderError>,
	body: Result<Vec<BodyPart>, VariationError>,
}

fn parse_game(scanner: &mut Peekable<impl Iterator<Item = PdnToken>>) -> Result<Game, GameError> {
	let header = parse_header(scanner);
	let body = parse_body_until(scanner, PdnTokenBody::Asterisk);
	whitespace_if_found(scanner);

	if let Ok(header) = header {
		if let Ok(body) = body {
			Ok(Game { header, body })
		} else {
			Err(GameError {
				header: Ok(header),
				body,
			})
		}
	} else {
		Err(GameError { header, body })
	}
}

pub type PdnError = Vec<Result<Game, GameError>>;

fn parse(scanner: &mut impl Iterator<Item = PdnToken>) -> Result<PdnFile, PdnError> {
	let mut scanner = scanner.peekable();
	let mut games = Vec::new();
	let mut game_separators = Vec::new();

	loop {
		let Some(token) = scanner.peek() else {
			break;
		};

		if token.body != PdnTokenBody::LeftBracket {
			break;
		}

		games.push(parse_game(&mut scanner));
		game_separators.push(scanner.next().unwrap().header);
	}

	if games.iter().any(|r| r.is_err()) {
		Err(games)
	} else {
		let games = games.iter().map(|r| r.as_ref().cloned().unwrap()).collect();
		Ok(PdnFile {
			games,
			game_separators,
		})
	}
}
