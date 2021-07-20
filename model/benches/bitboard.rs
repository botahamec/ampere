use criterion::{black_box, criterion_group, criterion_main, Criterion};
use model::CheckersBitBoard;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;

fn clone(c: &mut Criterion) {
	let board = CheckersBitBoard::starting_position();
	c.bench_function("clone", |b| b.iter(|| black_box(board.clone())));
}

fn hash(c: &mut Criterion) {
	let board = CheckersBitBoard::starting_position();
	let mut hasher = DefaultHasher::new();
	c.bench_function("hash", |b| b.iter(|| board.hash(black_box(&mut hasher))));
}

fn default(c: &mut Criterion) {
	c.bench_function("default", |b| {
		b.iter(|| black_box(CheckersBitBoard::default()))
	});
}

fn eq(c: &mut Criterion) {
	let board1 = CheckersBitBoard::default();
	let board2 = CheckersBitBoard::default();
	c.bench_function("equal", |b| {
		b.iter(|| black_box(board1) == black_box(board2))
	});
}

fn default_const(c: &mut Criterion) {
	c.bench_function("default (const)", |b| {
		b.iter(|| black_box(CheckersBitBoard::starting_position()))
	});
}

fn piece_at(c: &mut Criterion) {
	let board = CheckersBitBoard::starting_position();
	c.bench_function("piece", |b| b.iter(|| board.piece_at(black_box(0))));
}

fn color_at_unchecked(c: &mut Criterion) {
	let board = CheckersBitBoard::starting_position();
	c.bench_function("color (unsafe)", |b| {
		b.iter(|| unsafe { board.color_at_unchecked(black_box(1)) })
	});
}

fn king_at_unchecked(c: &mut Criterion) {
	let board = CheckersBitBoard::starting_position();
	c.bench_function("king (unsafe)", |b| {
		b.iter(|| unsafe { board.king_at_unchecked(black_box(2)) })
	});
}

fn color_at(c: &mut Criterion) {
	let board = CheckersBitBoard::starting_position();
	c.bench_function("color (safe - filled)", |b| {
		b.iter(|| board.color_at(black_box(3)))
	});

	c.bench_function("color (safe - empty)", |b| {
		b.iter(|| board.color_at(black_box(2)))
	});
}

fn king_at(c: &mut Criterion) {
	let board = CheckersBitBoard::starting_position();
	c.bench_function("king (safe - filled)", |b| {
		b.iter(|| board.king_at(black_box(4)))
	});

	c.bench_function("king (safe - empty)", |b| {
		b.iter(|| board.king_at(black_box(9)))
	});
}

criterion_group!(
	bitboard,
	clone,
	hash,
	eq,
	default,
	default_const,
	piece_at,
	color_at_unchecked,
	king_at_unchecked,
	color_at,
	king_at,
);
criterion_main!(bitboard);
