use std::sync::Arc;

use snob::{csets, csets::CharacterSet, Scanner};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
	White,
	Black,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PdnTokenBody {
	MoveNumber(usize, Color),
	MoveSeparator,
	CaptureSeparator,
	AlphaSquare(char, char),
	NumSquare(u8),
	MoveStrength(Arc<str>),
	Nag(usize),
	LeftParenthesis,
	RightParenthesis,
	LeftBracket,
	RightBracket,
	Asterisk,
	Setup(Arc<str>),
	String(Arc<str>),
	Comment(Arc<str>),
	Identifier(Arc<str>),
	Space(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenHeader {
	start: usize,
	len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdnToken {
	pub header: TokenHeader,
	pub body: PdnTokenBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenErrorType {
	InvalidNumber(usize),
	InvalidNag,
	InvalidSquare,
	UnterminatedSetup,
	UnterminatedComment,
	UnterminatedString,
	InvalidToken,
}

pub struct TokenError {
	header: TokenHeader,
	ty: TokenErrorType,
}

pub struct PdnScanner {
	scanner: Scanner,
}

impl PdnScanner {
	fn scan_string(&mut self) -> Option<String> {
		let mut string = String::new();
		loop {
			if let Some(position) = self.scanner.many("\\\"".complement()) {
				let part = self
					.scanner
					.goto(position)
					.expect("position should be valid");
				string.push_str(&part);
			} else if let Some(position) = self.scanner.starts_with("\\\"") {
				self.scanner.goto(position);
				string.push('"');
			} else {
				break;
			}
		}

		if let Some(position) = self.scanner.any('"') {
			self.scanner.goto(position);
			Some(string)
		} else {
			None
		}
	}

	fn scan_unescaped_string(&mut self, terminator: char) -> Option<String> {
		let position = self.scanner.upto(terminator)?;
		let string = self
			.scanner
			.goto(position)
			.expect("position should be valid");
		let position = self
			.scanner
			.any(terminator)
			.expect("there should be a terminator next");
		self.scanner.goto(position);
		Some(string)
	}

	fn scan_number(&mut self) -> Option<usize> {
		let position = self.scanner.many(csets::AsciiDigits)?;
		let number = self
			.scanner
			.goto(position)
			.expect("position should be valid");
		let number: usize = number.parse().expect("should be a valid number");
		Some(number)
	}

	fn scan_identifier(&mut self) -> Option<String> {
		let position = self
			.scanner
			.many(csets::AsciiLetters.union(csets::AsciiDigits).union('_'))?;
		let identifier = self
			.scanner
			.goto(position)
			.expect("position should be valid");
		Some(identifier)
	}

	fn next_token(&mut self) -> Option<Result<PdnTokenBody, TokenErrorType>> {
		if self.scanner.is_at_end() {
			return None;
		}

		let token = if let Some(position) = self.scanner.any('-') {
			self.scanner.goto(position);
			Ok(PdnTokenBody::MoveSeparator)
		} else if let Some(position) = self.scanner.any('x') {
			self.scanner.goto(position);
			Ok(PdnTokenBody::CaptureSeparator)
		} else if let Some(position) = self.scanner.any('(') {
			self.scanner.goto(position);

			// try a move strength token
			if let Some(position) = self.scanner.many("?!") {
				let char = self
					.scanner
					.char_at(position)
					.expect("position should be valid");
				if char == ')' {
					let strength = self
						.scanner
						.goto(position)
						.expect("position should be valid");
					let position = self
						.scanner
						.any(')')
						.expect("move strength should terminate");
					self.scanner.goto(position);
					return Some(Ok(PdnTokenBody::MoveStrength(strength.into())));
				}
			}

			Ok(PdnTokenBody::LeftParenthesis)
		} else if let Some(position) = self.scanner.any(')') {
			self.scanner.goto(position);
			Ok(PdnTokenBody::RightParenthesis)
		} else if let Some(position) = self.scanner.any('[') {
			self.scanner.goto(position);
			Ok(PdnTokenBody::LeftBracket)
		} else if let Some(position) = self.scanner.any(']') {
			self.scanner.goto(position);
			Ok(PdnTokenBody::RightBracket)
		} else if let Some(position) = self.scanner.any('*') {
			self.scanner.goto(position);
			Ok(PdnTokenBody::Asterisk)
		} else if let Some(position) = self.scanner.any('$') {
			self.scanner.goto(position);
			match self.scan_number() {
				Some(number) => Ok(PdnTokenBody::Nag(number)),
				None => Err(TokenErrorType::InvalidNag),
			}
		} else if let Some(position) = self.scanner.any('/') {
			self.scanner.goto(position);
			match self.scan_unescaped_string('/') {
				Some(string) => Ok(PdnTokenBody::Setup(string.into())),
				None => Err(TokenErrorType::UnterminatedSetup),
			}
		} else if let Some(position) = self.scanner.any('{') {
			self.scanner.goto(position);
			match self.scan_unescaped_string('}') {
				Some(string) => Ok(PdnTokenBody::Comment(string.into())),
				None => Err(TokenErrorType::UnterminatedComment),
			}
		} else if let Some(position) = self.scanner.any('"') {
			self.scanner.goto(position);
			match self.scan_string() {
				Some(string) => Ok(PdnTokenBody::String(string.into())),
				None => Err(TokenErrorType::UnterminatedString),
			}
		} else if let Some(position) = self.scanner.many("?!") {
			let strength = self
				.scanner
				.goto(position)
				.expect("position should be valid");
			Ok(PdnTokenBody::MoveStrength(strength.into()))
		} else if let Some(position) = self.scanner.any("abcdefgh") {
			let letter = self
				.scanner
				.goto(position)
				.expect("position should be valid")
				.chars()
				.next()
				.expect("should contain one letter");
			if let Some(position) = self.scanner.any("12345678") {
				let number = self
					.scanner
					.goto(position)
					.expect("position should be valid")
					.chars()
					.next()
					.expect("should contain one letter");
				Ok(PdnTokenBody::AlphaSquare(letter, number))
			} else {
				self.scanner.advance(1); // skip over second character
				Err(TokenErrorType::InvalidSquare)
			}
		} else if self.scanner.any(csets::AsciiUppercase).is_some() {
			let identifier = self
				.scan_identifier()
				.expect("should be a valid identifier");
			Ok(PdnTokenBody::Identifier(identifier.into()))
		} else if self.scanner.any(csets::AsciiDigits).is_some() {
			let number = self.scan_number().expect("should be a valid number");
			if let Some(position) = self.scanner.starts_with("...") {
				self.scanner.goto(position);
				Ok(PdnTokenBody::MoveNumber(number, Color::Black))
			} else if let Some(position) = self.scanner.any('.') {
				self.scanner.goto(position);
				Ok(PdnTokenBody::MoveNumber(number, Color::White))
			} else if number < 100 {
				Ok(PdnTokenBody::NumSquare(number as u8))
			} else {
				Err(TokenErrorType::InvalidNumber(number))
			}
		} else if let Some(position) = self.scanner.many(csets::AsciiWhitespace) {
			let whitespace = self
				.scanner
				.goto(position)
				.expect("position should be valid");
			Ok(PdnTokenBody::Space(whitespace.into()))
		} else {
			let position = self
				.scanner
				.upto(csets::AsciiLetters.union(csets::AsciiDigits.union("-x(?!)[]")))
				.unwrap_or_else(|| self.scanner.len());

			self.scanner
				.goto(position)
				.expect("position should be valid");

			Err(TokenErrorType::InvalidToken)
		};

		Some(token)
	}
}

impl Iterator for PdnScanner {
	type Item = Result<PdnToken, TokenError>;

	fn next(&mut self) -> Option<Self::Item> {
		let start = self.scanner.position();
		let token = self.next_token()?;
		let end = self.scanner.position();
		let len = end - start;
		let header = TokenHeader { start, len };

		let token = match token {
			Ok(token) => Ok(PdnToken {
				header,
				body: token,
			}),
			Err(error) => Err(TokenError { header, ty: error }),
		};

		Some(token)
	}
}
