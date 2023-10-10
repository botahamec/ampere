use std::{io, string::FromUtf8Error};

use byteorder::{BigEndian, ReadBytesExt};
use model::{CheckersBitBoard, PieceColor};
use thiserror::Error;

const MAGIC: u32 = u32::from_be_bytes(*b".amp");
const SUPPORTED_VERSION: u16 = 0;
const MAX_TABLE_LENGTH: u64 = 5_000_000_000;

#[derive(Debug, Clone, PartialEq)]
pub struct Tablebase {
	header: FileHeader,
	entries: Box<[Option<TablebaseEntry>]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileHeader {
	/// The version of Ampere Tablebase Format being used
	version: u16,
	/// The magic number multiplied by board hash values
	magic_factor: u64,
	/// The number of entries in the tablebase
	entries_count: u64,
	/// The length of the table needed in-memory
	table_length: u64,
	/// The type of game the tablebase is for
	game_type: GameType,
	/// The name of the tablebase
	tablebase_name: Box<str>,
	/// The tablebase author
	author_name: Box<str>,
	/// The Unix timestamp indicating when the tablebase was created
	publication_time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GameType {
	/// The type of game being played
	game_type: Game,
	/// The color that makes the first move
	start_color: PieceColor,
	/// The width of the board
	board_width: u8,
	/// The height of the board
	board_height: u8,
	/// The move notation
	notation: MoveNotation,
	/// True if the bottom-left square is a playing square
	invert_flag: bool,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Game {
	EnglishDraughts = 21,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveNotation {
	/// Standard Chess Notation, like e5
	Standard = 0,
	/// Alpha-numeric square representation, like e7-e5
	Alpha = 1,
	/// Numeric square representation, like 11-12
	Numeric = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TablebaseEntry {
	board: CheckersBitBoard,
	evaluation: f32,
	depth: u8,
}

#[derive(Debug, Error)]
enum TablebaseFileError {
	#[error("Invalid tablebase: the magic header field was incorrect")]
	MagicError,
	#[error("This version of the tablebase format is unsupported. Only {SUPPORTED_VERSION} is supported")]
	UnsupportedVersion(u16),
	#[error("The table is too large. The length of the table is {} entries, but the max is only {}", .found, .max)]
	TableTooLarge { found: u64, max: u64 },
	#[error("The game type for this tablebase is unsupported. Only standard American Checkers is supported")]
	UnsupportedGameType(u8),
	#[error("A string was not valid UTF-8: {}", .0)]
	InvalidString(#[from] FromUtf8Error),
	#[error(transparent)]
	IoError(#[from] io::Error),
}

fn read_header(reader: &mut impl ReadBytesExt) -> Result<FileHeader, TablebaseFileError> {
	// magic is used to verify that the file is valid
	let magic = reader.read_u32::<BigEndian>()?;
	if magic != MAGIC {
		return Err(TablebaseFileError::MagicError);
	}

	read_reserved_bytes::<2>(reader)?;

	let version = reader.read_u16::<BigEndian>()?;
	if version != SUPPORTED_VERSION {
		return Err(TablebaseFileError::UnsupportedVersion(version));
	}

	let magic_factor = reader.read_u64::<BigEndian>()?;
	let entries_count = reader.read_u64::<BigEndian>()?;
	let table_length = reader.read_u64::<BigEndian>()?;

	if table_length > MAX_TABLE_LENGTH {
		return Err(TablebaseFileError::TableTooLarge {
			found: table_length,
			max: MAX_TABLE_LENGTH,
		});
	}

	let game_type = read_game_type(reader)?;
	let publication_time = reader.read_u64::<BigEndian>()?;
	let tablebase_name_len = reader.read_u8()?;
	let author_name_len = reader.read_u8()?;
	let _ = read_reserved_bytes::<14>(reader);

	let tablebase_name = read_string(reader, tablebase_name_len)?;
	let author_name = read_string(reader, author_name_len)?;

	Ok(FileHeader {
		version,
		magic_factor,
		entries_count,
		table_length,
		game_type,
		publication_time,
		tablebase_name,
		author_name,
	})
}

fn read_reserved_bytes<const NUM_BYTES: usize>(reader: &mut impl ReadBytesExt) -> io::Result<()> {
	reader.read_exact([0; NUM_BYTES].as_mut_slice())?;
	Ok(())
}

#[derive(Debug, Error)]
enum ReadStringError {
	#[error(transparent)]
	InvalidUtf8(#[from] FromUtf8Error),
	#[error(transparent)]
	IoError(#[from] io::Error),
}

fn read_string(reader: &mut impl ReadBytesExt, len: u8) -> Result<Box<str>, TablebaseFileError> {
	let mut buffer = vec![0; len as usize];
	reader.read_exact(&mut buffer)?;
	Ok(String::from_utf8(buffer)?.into_boxed_str())
}

fn read_game_type(reader: &mut impl ReadBytesExt) -> Result<GameType, TablebaseFileError> {
	read_reserved_bytes::<1>(reader)?;
	let game_type = reader.read_u8()?;
	let start_color = reader.read_u8()?;
	let board_width = reader.read_u8()?;
	let board_height = reader.read_u8()?;
	let invert_flag = reader.read_u8()?;
	let notation = reader.read_u8()?;
	read_reserved_bytes::<1>(reader)?;

	if game_type != 21
		|| start_color != 1
		|| board_width != 8
		|| board_height != 8
		|| invert_flag != 1
		|| notation != 2
	{
		Err(TablebaseFileError::UnsupportedGameType(game_type))
	} else {
		Ok(GameType {
			game_type: Game::EnglishDraughts,
			start_color: PieceColor::Dark,
			board_width: 8,
			board_height: 8,
			notation: MoveNotation::Numeric,
			invert_flag: true,
		})
	}
}
