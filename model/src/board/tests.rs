use std::collections::hash_map::DefaultHasher;

use proptest::prelude::*;

use super::*;

proptest! {
	#[test]
	fn test_bitboard_new(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX) {
		let board = CheckersBitBoard::new(p, c, k, PieceColor::Dark);
		assert_eq!(p, board.pieces);
		assert_eq!(c, board.color);
		assert_eq!(k, board.kings);
	}

	#[test]
	fn test_bits_fns(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		assert_eq!(p, board.pieces_bits());
		assert_eq!(c, board.color_bits());
		assert_eq!(k, board.king_bits());
	}

	#[test]
	fn test_bitboard_hash(pieces in 0u32..=u32::MAX, color in 0u32..=u32::MAX, kings in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX) {
		let board1 = CheckersBitBoard {
			pieces, color, kings, turn: PieceColor::Dark
		};
		let board2 = CheckersBitBoard {
			pieces,
			color: c,
			kings: k,
			turn: PieceColor::Dark
		};
		let mut hasher1 = DefaultHasher::new();
		let mut hasher2 = DefaultHasher::new();
		board1.hash(&mut hasher1);
		board2.hash(&mut hasher2);
		assert_eq!(hasher1.finish(), hasher2.finish());
	}

	#[test]
	fn test_bitboard_eq_identical(pieces in 0u32..=u32::MAX, color in 0u32..u32::MAX, kings in 0u32..=u32::MAX) {
		let board1 = CheckersBitBoard {pieces, color, kings, turn: PieceColor::Dark};
		let board2 = CheckersBitBoard {pieces, color, kings, turn: PieceColor::Dark};
		assert_eq!(board1, board2);
	}

	#[test]
	fn test_bitboard_eq_empty(c1 in 0u32..u32::MAX, k1 in 0u32..=u32::MAX, c2 in 0u32..u32::MAX, k2 in 0u32..=u32::MAX) {
		let board1 = CheckersBitBoard {pieces: 0, color: c1, kings: k1, turn: PieceColor::Dark};
		let board2 = CheckersBitBoard {pieces: 0, color: c2, kings: k2, turn: PieceColor::Dark};
		assert_eq!(board1, board2);
	}

	#[test]
	fn test_piece_at(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX, v in 0usize..32) {
		let board = CheckersBitBoard {
			pieces: p,
			color: c,
			kings: k,
			turn: PieceColor::Dark
		};

		// just test for no crash
		let _ = board.piece_at(v);
	}

	#[test]
	fn test_color_at_unchecked(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX, v in 0usize..32) {
		let board = CheckersBitBoard {
			pieces: p,
			color: c,
			kings: k,
			turn: PieceColor::Dark
		};

		// just test for no crash
		unsafe {let _ = board.color_at_unchecked(v);}
	}

	#[test]
	fn test_king_at_unchecked(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX, v in 0usize..32) {
		let board = CheckersBitBoard {
			pieces: p,
			color: c,
			kings: k,
			turn: PieceColor::Dark
		};
		unsafe {let _ = board.king_at_unchecked(v);}
	}

	#[test]
	fn test_color_at(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX, v in 0usize..32) {
		let board = CheckersBitBoard {
			pieces: p,
			color: c,
			kings: k,
			turn: PieceColor::Dark
		};

		// just testing for no crash
		let _  = board.color_at(v);
	}

	#[test]
	fn test_king_at(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX, v in 0usize..32) {
		let board = CheckersBitBoard {
			pieces: p,
			color: c,
			kings: k,
			turn: PieceColor::Dark
		};

		// just testing for no crash
		let _ = board.king_at(v);
	}

	#[test]
	fn test_move_piece_to(p in 0u32..=u32::MAX, c in 0u32..=u32::MAX, k in 0u32..=u32::MAX, s in 0usize..32, e in 0usize..32) {
		let board = CheckersBitBoard {
			pieces: p,
			color: c,
			kings: k,
			turn: PieceColor::Dark
		};
		unsafe {board.move_piece_to_unchecked(s, e)};
	}

	#[test]
	fn test_move_forward(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX, v in 0usize..32, a in 0usize..usize::MAX) {
		if a <= usize::MAX - v { // so there's no overflow
			let board = CheckersBitBoard {
				pieces: p, color: c, kings: k, turn: PieceColor::Dark
			};
			unsafe {board.move_piece_forward_unchecked(v, a)};
		}
	}

	#[test]
	fn test_move_backward(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX, v in 0usize..32, a in 0usize..usize::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};
		unsafe {board.move_piece_backward_unchecked(v, a)};
	}

	#[test]
	fn test_move_forward_left(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		if board.piece_at(0) {
			let board2 = unsafe {board.move_piece_forward_left_unchecked(0)};
			assert_eq!(board2.color_at(7), board.color_at(0));
			assert_eq!(board2.king_at(7), board.king_at(0));
		}
	}

	#[test]
	fn test_move_forward_right(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		if board.piece_at(18) {
			let board2 = unsafe {board.move_piece_forward_right_unchecked(18)};
			assert_eq!(board2.color_at(19), board.color_at(18));
			assert_eq!(board2.king_at(19), board.king_at(18));
		}
	}

	#[test]
	fn test_move_backward_left(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		if board.piece_at(25) {
			let board2 = unsafe {board.move_piece_backward_left_unchecked(25)};
			assert_eq!(board2.color_at(24), board.color_at(25));
			assert_eq!(board2.king_at(24), board.king_at(25));
		}
	}

	#[test]
	fn test_move_backward_right(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};
		if board.piece_at(11) {
			let board2 = unsafe {board.move_piece_backward_right_unchecked(11)};
			assert_eq!(board2.color_at(4), board.color_at(11));
			assert_eq!(board2.king_at(4), board.king_at(11));
		}
	}

	#[test]
	fn test_clear_piece(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX, v in 0usize..32) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		let board = board.clear_piece(v);
		assert!(!board.piece_at(v));
	}

	#[test]
	fn test_jump_forward_left(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		unsafe {
			if board.piece_at(0) && board.piece_at(7) && !board.piece_at(14) && board.color_at_unchecked(0) != board.color_at_unchecked(7) {
				let board2 = board.jump_piece_forward_left_unchecked(0);
				assert!(!board2.piece_at(0));
				assert!(!board2.piece_at(7));
				assert!(board2.piece_at(14));
				assert_eq!(board2.color_at_unchecked(14), board.color_at_unchecked(0));
				assert_eq!(board2.king_at_unchecked(14), board.king_at_unchecked(0));
			}
		}
	}

	#[test]
	fn test_jump_forward_right(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		unsafe {
			if board.piece_at(18) && board.piece_at(19) && !board.piece_at(20) && board.color_at_unchecked(18) != board.color_at_unchecked(19) {
				let board2 = board.jump_piece_forward_right_unchecked(18);
				assert!(!board2.piece_at(18));
				assert!(!board2.piece_at(19));
				assert!(board2.piece_at(20));
				assert_eq!(board2.color_at_unchecked(20), board.color_at_unchecked(18));
				assert_eq!(board2.king_at_unchecked(20), board.king_at_unchecked(18));
			}
		}
	}

	#[test]
	fn test_jump_backward_left(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		unsafe {
			if board.piece_at(25) && board.piece_at(24) && !board.piece_at(23) && board.color_at_unchecked(25) != board.color_at_unchecked(24) {
				let board2 = board.jump_piece_backward_left_unchecked(25);
				assert!(!board2.piece_at(25));
				assert!(!board2.piece_at(24));
				assert!(board2.piece_at(23));
				assert_eq!(board2.color_at_unchecked(23), board.color_at_unchecked(25));
				assert_eq!(board2.king_at_unchecked(23), board.king_at_unchecked(25));
			}
		}
	}

	#[test]
	fn test_jump_backward_right(p in 0..u32::MAX, c in 0..u32::MAX, k in 0..u32::MAX) {
		let board = CheckersBitBoard {
			pieces: p, color: c, kings: k, turn: PieceColor::Dark
		};

		unsafe {
			if board.piece_at(11) && board.piece_at(4) && !board.piece_at(29) && board.color_at_unchecked(11) != board.color_at_unchecked(4) {
				let board2 = board.jump_piece_backward_right_unchecked(11);
				assert!(!board2.piece_at(11));
				assert!(!board2.piece_at(4));
				assert!(board2.piece_at(29));
				assert_eq!(board2.color_at_unchecked(29), board.color_at_unchecked(11));
				assert_eq!(board2.king_at_unchecked(29), board.king_at_unchecked(11));
			}
		}
	}
}

#[test]
fn test_piece_at_empty_board() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0,
		kings: 0,
		turn: PieceColor::Dark,
	};

	// There should be no piece in any space
	for i in 0..32 {
		assert!(!board.piece_at(i))
	}
}

#[test]
fn test_piece_at_space_zero() {
	let board = CheckersBitBoard {
		pieces: 1,
		color: 0,
		kings: 0,
		turn: PieceColor::Dark,
	};
	assert!(board.piece_at(0)); // There should be a piece in space 0

	// There should be no piece in any other square
	for i in 1..32 {
		assert!(!board.piece_at(i))
	}
}

#[test]
fn test_color_at_unchecked_all_light() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0,
		kings: 0,
		turn: PieceColor::Dark,
	};

	// All squares should be light
	for i in 0..32 {
		assert_eq!(unsafe { board.color_at_unchecked(i) }, PieceColor::Light)
	}
}

#[test]
fn test_color_at_unchecked_all_dark() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: u32::MAX,
		kings: 0,
		turn: PieceColor::Dark,
	};

	// All squares should be dark
	for i in 0..32 {
		assert_eq!(unsafe { board.color_at_unchecked(i) }, PieceColor::Dark)
	}
}

#[test]
fn test_king_at_unchecked_all_kings() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0,
		kings: u32::MAX,
		turn: PieceColor::Dark,
	};

	// All squares should be kings
	for i in 0..32 {
		assert!(unsafe { board.king_at_unchecked(i) })
	}
}

#[test]
fn test_king_at_unchecked_one_king() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0,
		kings: 1,
		turn: PieceColor::Dark,
	};

	assert!(unsafe { board.king_at_unchecked(0) });

	// All other squares should be peasants
	for i in 1..32 {
		assert!(!unsafe { board.king_at_unchecked(i) })
	}
}

#[test]
fn test_default_bitboard() {
	let board = CheckersBitBoard::default();
	let exemptions = vec![2, 28, 22, 16, 27, 21, 15, 9];
	let black = vec![18, 12, 6, 0, 19, 13, 7, 1, 26, 20, 14, 8];

	for i in 0..32 {
		if !exemptions.contains(&i) {
			assert!(board.piece_at(i));
			assert!(!unsafe { board.king_at_unchecked(i) });

			if black.contains(&i) {
				assert_eq!(unsafe { board.color_at_unchecked(i) }, PieceColor::Dark)
			} else {
				assert_eq!(unsafe { board.color_at_unchecked(i) }, PieceColor::Light)
			}
		} else {
			assert!(!board.piece_at(i))
		}
	}
}

#[test]
fn test_bitboard_eq_default() {
	let board1 = CheckersBitBoard {
		pieces: 0b11100111100111100111110111111011,
		color: 0b11110011110000110000110000111100,
		kings: 0,
		turn: PieceColor::Dark,
	};
	let board2 = CheckersBitBoard {
		pieces: 0b11100111100111100111110111111011,
		color: 0b11110011110000110000110000111100,
		kings: 0,
		turn: PieceColor::Dark,
	};
	assert_eq!(board1, board2);
}

#[test]
fn test_bitboard_neq_color() {
	let board1 = CheckersBitBoard {
		pieces: 0b11100111100111100111110111111011,
		color: 0b11110011110000110000110000111100,
		kings: 0,
		turn: PieceColor::Dark,
	};
	let board2 = CheckersBitBoard {
		pieces: 0b11100111100111100111110111111011,
		color: 465413646,
		kings: 0,
		turn: PieceColor::Dark,
	};
	assert_ne!(board1, board2);
}

#[test]
fn test_bitboard_neq_kings() {
	let board1 = CheckersBitBoard {
		pieces: 0b11100111100111100111110111111011,
		color: 0b11110011110000110000110000111100,
		kings: 0,
		turn: PieceColor::Dark,
	};
	let board2 = CheckersBitBoard {
		pieces: 0b11100111100111100111110111111011,
		color: 0b11110011110000110000110000111100,
		kings: 465413646,
		turn: PieceColor::Dark,
	};
	assert_ne!(board1, board2);
}

#[test]
fn test_color_at_empty() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0,
		kings: 0,
		turn: PieceColor::Dark,
	};

	for i in 0..32 {
		assert_eq!(board.color_at(i), None)
	}
}

#[test]
fn test_color_at_specified_empty_colors() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0b01,
		kings: 0,
		turn: PieceColor::Dark,
	};

	for i in 0..32 {
		assert_eq!(board.color_at(i), None)
	}
}

#[test]
fn test_color_at_some_colors() {
	let board = CheckersBitBoard {
		pieces: 3,
		color: 0b01,
		kings: 0,
		turn: PieceColor::Dark,
	};

	assert_eq!(board.color_at(0), Some(PieceColor::Dark));
	assert_eq!(board.color_at(1), Some(PieceColor::Light));

	for i in 2..32 {
		assert_eq!(board.color_at(i), None)
	}
}

#[test]
fn test_king_at_empty() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0,
		kings: 0,
		turn: PieceColor::Dark,
	};

	for i in 0..32 {
		assert_eq!(board.king_at(i), None)
	}
}

#[test]
fn test_king_at_specified_empty_colors() {
	let board = CheckersBitBoard {
		pieces: 0,
		color: 0,
		kings: 0b01,
		turn: PieceColor::Dark,
	};

	for i in 0..32 {
		assert_eq!(board.king_at(i), None)
	}
}

#[test]
fn test_king_at_some_colors() {
	let board = CheckersBitBoard {
		pieces: 3,
		color: 0,
		kings: 0b01,
		turn: PieceColor::Dark,
	};

	assert_eq!(board.king_at(0), Some(true));
	assert_eq!(board.king_at(1), Some(false));

	for i in 2..32 {
		assert_eq!(board.king_at(i), None)
	}
}

#[test]
fn test_move_piece_to_default_board() {
	let board = CheckersBitBoard::default();
	let board = unsafe { board.move_piece_to_unchecked(0, 5) };
	assert!(!board.piece_at(0));
	assert!(board.piece_at(5));
	assert_eq!(board.color_at(5).unwrap(), PieceColor::Dark);
	assert!(board.king_at(5).unwrap());
	assert_eq!(board.turn, PieceColor::Light);
}

#[test]
fn test_move_piece_forward_standard() {
	let board = CheckersBitBoard::default();
	let board = unsafe { board.move_piece_forward_unchecked(14, 2) }; // go to 16
	assert!(!board.piece_at(14));
	assert!(board.piece_at(16));
	assert_eq!(board.color_at(16).unwrap(), PieceColor::Dark);
	assert!(!board.king_at(16).unwrap());
	assert_eq!(board.turn, PieceColor::Light);
}

#[test]
fn test_move_piece_forward_wrap() {
	let board = CheckersBitBoard::default();
	let board = unsafe { board.move_piece_forward_unchecked(26, 8) }; // go to 9
	assert!(!board.piece_at(26));
	assert!(board.piece_at(2));
	assert_eq!(board.color_at(2).unwrap(), PieceColor::Dark);
	assert!(!board.king_at(2).unwrap());
	assert_eq!(board.turn, PieceColor::Light);
}

#[test]
fn test_move_piece_forward_left_to_king() {
	let board = CheckersBitBoard::new(0b10000, 0b10000, 0, PieceColor::Dark);
	let board = unsafe { board.move_piece_forward_left_unchecked(4) };
	assert!(board.piece_at(11));
	assert!(board.king_at(11).unwrap());
}

#[test]
fn test_move_piece_backward_left_to_king() {
	let board = CheckersBitBoard::new(0b10, 0, 0, PieceColor::Dark);
	let board = unsafe { board.move_piece_backward_left_unchecked(1) };
	assert!(board.piece_at(0));
	assert!(board.king_at(0).unwrap());
}

#[test]
fn test_move_piece_backward_standard() {
	let board = CheckersBitBoard::default().flip_turn();
	let board = unsafe { board.move_piece_backward_unchecked(29, 14) }; // go to 15
	assert!(!board.piece_at(29));
	assert!(board.piece_at(15));
	assert_eq!(board.color_at(15).unwrap(), PieceColor::Light);
	assert!(!board.king_at(15).unwrap());
	assert_eq!(board.turn, PieceColor::Dark);
}

#[test]
fn test_move_piece_backward_wrap() {
	let board = CheckersBitBoard::default();
	let board = unsafe { board.move_piece_backward_unchecked(0, 4) }; // go to 28
	assert!(!board.piece_at(0));
	assert!(board.piece_at(28));
	assert_eq!(board.color_at(28).unwrap(), PieceColor::Dark);
	assert!(!board.king_at(28).unwrap());
	assert_eq!(board.turn, PieceColor::Light);
}

#[test]
// the specific tests have special values, and are different from the property tests
fn test_jump_forward_left_specific() {
	let board = CheckersBitBoard {
		pieces: 0b10000001,
		color: 1,
		kings: 0,
		turn: PieceColor::Dark,
	};

	let board2 = unsafe { board.jump_piece_forward_left_unchecked(0) };
	assert!(!board2.piece_at(0));
	assert!(!board2.piece_at(7));
	assert!(board2.piece_at(14));
	assert_eq!(board2.color_at(14).unwrap(), board.color_at(0).unwrap());
	assert_eq!(board2.king_at(14).unwrap(), board.king_at(0).unwrap());
	assert_eq!(board2.turn, PieceColor::Light);
}

#[test]
fn test_jump_forward_right_specific() {
	let board = CheckersBitBoard {
		pieces: 0b11000000000000000000,
		color: 0b10000000000000000000,
		kings: 0,
		turn: PieceColor::Dark,
	};

	let board2 = unsafe { board.jump_piece_forward_right_unchecked(18) };
	assert!(!board2.piece_at(18));
	assert!(!board2.piece_at(19));
	assert!(board2.piece_at(20));
	assert_eq!(board2.color_at(20).unwrap(), board.color_at(18).unwrap());
	assert_eq!(board2.king_at(20).unwrap(), board.king_at(18).unwrap());
	assert_eq!(board2.turn, PieceColor::Light);
}

#[test]
fn test_jump_backward_left_specific() {
	let board = CheckersBitBoard {
		pieces: 0b110000000000000000000000000,
		color: 0b100000000000000000000000000,
		kings: 0,
		turn: PieceColor::Dark,
	};

	let board2 = unsafe { board.jump_piece_backward_left_unchecked(25) };
	assert!(!board2.piece_at(25));
	assert!(!board2.piece_at(24));
	assert!(board2.piece_at(23));
	assert_eq!(board2.color_at(23).unwrap(), board.color_at(25).unwrap());
	assert_eq!(board2.king_at(23).unwrap(), board.king_at(25).unwrap());
	assert_eq!(board2.turn, PieceColor::Light);
}

#[test]
fn test_jump_backward_right_specific() {
	let board = CheckersBitBoard {
		pieces: 0b100000010000,
		color: 0b10000,
		kings: 0,
		turn: PieceColor::Dark,
	};

	let board2 = unsafe { board.jump_piece_backward_right_unchecked(11) };
	assert!(!board2.piece_at(11));
	assert!(!board2.piece_at(4));
	assert!(board2.piece_at(29));
	assert_eq!(board2.color_at(29).unwrap(), board.color_at(11).unwrap());
	assert_eq!(board2.king_at(29).unwrap(), board.king_at(11).unwrap());
	assert_eq!(board2.turn, PieceColor::Light);
}

#[test]
fn test_send() {
	fn assert_send<T: Send>() {}
	assert_send::<CheckersBitBoard>();
}

#[test]
fn test_sync() {
	fn assert_sync<T: Sync>() {}
	assert_sync::<CheckersBitBoard>();
}
