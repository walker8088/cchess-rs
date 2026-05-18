// Integration tests for cchess-rs

use cchess_rs::board::Board;
use cchess_rs::game::Game;
use cchess_rs::move_gen::generate_moves;
use cchess_rs::pieces::{Color, PieceType};

#[test]
fn test_color_opposite() {
    assert_eq!(Color::Red.opposite(), Color::Black);
    assert_eq!(Color::Black.opposite(), Color::Red);
}

#[test]
fn test_color_from_fen() {
    assert_eq!(Color::from_fen('k'), Some(Color::Red));
    assert_eq!(Color::from_fen('K'), Some(Color::Black));
    assert_eq!(Color::from_fen('.'), None);
    assert_eq!(Color::from_fen('r'), Some(Color::Red));
    assert_eq!(Color::from_fen('R'), Some(Color::Black));
}

#[test]
fn test_piece_type_from_fen() {
    assert_eq!(PieceType::from_fen('k'), Some(PieceType::King));
    assert_eq!(PieceType::from_fen('a'), Some(PieceType::Advisor));
    assert_eq!(PieceType::from_fen('b'), Some(PieceType::Elephant));
    assert_eq!(PieceType::from_fen('n'), Some(PieceType::Knight));
    assert_eq!(PieceType::from_fen('r'), Some(PieceType::Rook));
    assert_eq!(PieceType::from_fen('c'), Some(PieceType::Cannon));
    assert_eq!(PieceType::from_fen('p'), Some(PieceType::Pawn));
    assert_eq!(PieceType::from_fen('.'), None);
    assert_eq!(PieceType::from_fen('x'), None);
}

#[test]
fn test_board_creation() {
    let board = Board::new();

    // Check that board is not empty
    assert!(!board.is_empty());

    // Check number of pieces (should be 32 in standard setup)
    assert_eq!(board.count_pieces(), 32);

    // Check red and black pieces count (should be 16 each)
    assert_eq!(board.count_color_pieces(true), 16); // Red pieces
    assert_eq!(board.count_color_pieces(false), 16); // Black pieces
}

#[test]
fn test_board_to_fen() {
    let board = Board::new();
    let fen = board.to_fen();

    // Standard starting FEN for Chinese Chess
    let expected_start = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";
    assert!(fen.starts_with(expected_start));
}

#[test]
fn test_board_from_fen() {
    let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";
    let board = Board::from_fen(fen).expect("Failed to parse FEN");

    // Verify board was created
    assert!(!board.is_empty());
    assert_eq!(board.count_pieces(), 32);
}

#[test]
fn test_fen_error_handling() {
    // Test invalid FEN (too few rows)
    let invalid_fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9";
    assert!(Board::from_fen(invalid_fen).is_err());

    // Test invalid FEN (too many rows)
    let invalid_fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR/extra";
    assert!(Board::from_fen(invalid_fen).is_err());
}

#[test]
fn test_board_clear() {
    let mut board = Board::new();
    board.clear();

    assert!(board.is_empty());
    assert_eq!(board.count_pieces(), 0);
}

#[test]
fn test_board_make_move() {
    let mut board = Board::new();

    // Test a valid move: red pawn at (0,3) moves forward to (0,4)
    let from = (0, 3);
    let to = (0, 4);

    assert!(board.make_move(from, to));

    // Verify the move was made
    assert!(board.is_empty_at(0, 3)); // From position should be empty
    assert!(!board.is_empty_at(0, 4)); // To position should have a piece
}

#[test]
fn test_board_helper_functions() {
    let _board = Board::new();

    // Test is_within_bounds
    assert!(Board::is_within_bounds(0, 0));
    assert!(Board::is_within_bounds(8, 9));
    assert!(!Board::is_within_bounds(9, 0)); // Column out of bounds
    assert!(!Board::is_within_bounds(0, 10)); // Row out of bounds

    // Test is_in_palace
    assert!(Board::is_in_palace(4, 0, true)); // Red palace
    assert!(Board::is_in_palace(4, 9, false)); // Black palace
    assert!(!Board::is_in_palace(0, 0, true)); // Not in palace

    // Test is_across_river
    assert!(!Board::is_across_river(3, true)); // Red side, not across river
    assert!(Board::is_across_river(5, true)); // Red side, across river
    assert!(!Board::is_across_river(7, false)); // Black side, not across river
    assert!(Board::is_across_river(4, false)); // Black side, across river
}

#[test]
fn test_board_methods() {
    let board = Board::new();

    // Test get_all_piece_positions
    let positions = board.get_all_piece_positions();
    assert_eq!(positions.len(), 32);

    // Test get_color_piece_positions
    let red_positions = board.get_color_piece_positions(true);
    let black_positions = board.get_color_piece_positions(false);
    assert_eq!(red_positions.len(), 16);
    assert_eq!(black_positions.len(), 16);

    // Test copy and equals
    let copy = board.copy();
    assert!(board.equals(&copy));
}

#[test]
fn test_game_creation() {
    let game = Game::new();

    assert_eq!(game.current_turn, Color::Red);
    assert!(!game.is_game_over);
    assert!(game.winner.is_none());
    assert!(game.move_history.is_empty());
}

#[test]
fn test_move_generation() {
    let board = Board::new();

    // Generate moves for red
    let red_moves = generate_moves(&board, Color::Red);
    assert!(!red_moves.is_empty());

    // Generate moves for black
    let black_moves = generate_moves(&board, Color::Black);
    assert!(!black_moves.is_empty());
}
