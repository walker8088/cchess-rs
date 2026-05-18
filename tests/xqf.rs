/// Tests for XQF file format support
use cchess_rs::board::Board;
use cchess_rs::game::Game;
use cchess_rs::xqf::{board_from_xqf, board_to_xqf, XqfFile, XqfGameInfo, XqfHeader, XqfMove};

#[test]
fn test_xqf_header_creation() {
    let header = XqfHeader::new();
    assert_eq!(header.signature, [b'X', b'Q', b'$', b'!']);
    assert_eq!(header.version, 0x0100);
}

#[test]
fn test_xqf_game_info_creation() {
    let game_info = XqfGameInfo::new();
    assert!(game_info.title.is_empty());
    assert!(game_info.red_player.is_empty());
    assert!(game_info.black_player.is_empty());
    assert_eq!(game_info.result, 0);
}

#[test]
fn test_xqf_move_creation() {
    let mv = XqfMove::new(10, 20, 1, 0);
    assert_eq!(mv.from, 10);
    assert_eq!(mv.to, 20);
    assert_eq!(mv.piece_type, 1);
    assert_eq!(mv.flags, 0);
}

#[test]
fn test_xqf_move_coordinates_conversion() {
    // Test position 0 (col 0, row 0)
    let (col, row) = XqfMove::to_coordinates(0);
    assert_eq!(col, 0);
    assert_eq!(row, 0);

    // Test position 10 (col 1, row 1)
    let (col, row) = XqfMove::to_coordinates(10);
    assert_eq!(col, 1);
    assert_eq!(row, 1);

    // Test position 89 (col 8, row 9) - bottom right corner
    let (col, row) = XqfMove::to_coordinates(89);
    assert_eq!(col, 8);
    assert_eq!(row, 9);

    // Test conversion back
    let pos = XqfMove::from_coordinates(4, 5);
    assert_eq!(pos, 49); // 5 * 9 + 4 = 49
}

#[test]
fn test_board_xqf_conversion() {
    // Create a standard board
    let board = Board::new();

    // Convert to XQF format
    let xqf_data = board_to_xqf(&board);
    assert!(xqf_data.is_ok());

    let data = xqf_data.unwrap();

    // Check that we have 90 bytes
    assert_eq!(data.len(), 90);

    // Check some specific positions
    // Red king at (4, 0) should be code 1
    let red_king_pos = 0 * 9 + 4;
    assert_eq!(data[red_king_pos], 1);

    // Red rook at (0, 0) should be code 5
    let red_rook_pos = 0 * 9 + 0;
    assert_eq!(data[red_rook_pos], 5);

    // Black king at (4, 9) should be code 9
    let black_king_pos = 9 * 9 + 4;
    assert_eq!(data[black_king_pos], 9);

    // Black rook at (0, 9) should be code 13
    let black_rook_pos = 9 * 9 + 0;
    assert_eq!(data[black_rook_pos], 13);
}

#[test]
fn test_board_from_xqf() {
    // Create a simple XQF board data
    let mut data = [0u8; 90];

    // Place red king at (4, 0)
    data[4] = 1;

    // Place black king at (4, 9)
    data[9 * 9 + 4] = 9;

    // Convert from XQF
    let board_result = board_from_xqf(&data);
    assert!(board_result.is_ok());

    let board = board_result.unwrap();

    // Check that positions are correctly set
    assert!(board.get_piece_at(4, 0).is_some());
    assert!(board.get_piece_at(4, 9).is_some());

    // Check piece types
    if let Some((piece_type, color)) = board.get_piece_at(4, 0) {
        use cchess_rs::pieces::{Color, PieceType};
        assert_eq!(piece_type, PieceType::King);
        assert_eq!(color, Color::Red);
    }

    if let Some((piece_type, color)) = board.get_piece_at(4, 9) {
        use cchess_rs::pieces::{Color, PieceType};
        assert_eq!(piece_type, PieceType::King);
        assert_eq!(color, Color::Black);
    }
}

#[test]
fn test_xqf_file_creation() {
    let game = Game::new();

    let xqf_file = XqfFile::from_game(&game, "Test Game", "Red Player", "Black Player");

    assert!(xqf_file.is_ok());

    let file = xqf_file.unwrap();
    assert_eq!(file.game_info.title, "Test Game");
    assert_eq!(file.game_info.red_player, "Red Player");
    assert_eq!(file.game_info.black_player, "Black Player");
    assert!(file.initial_board.is_some());
}

#[test]
fn test_board_get_set_piece() {
    let mut board = Board::new();
    board.clear();

    use cchess_rs::pieces::{Color, PieceType};

    // Set a red king at (4, 0)
    board.set_piece_at(4, 0, PieceType::King, Color::Red);

    // Get the piece back
    let piece = board.get_piece_at(4, 0);
    assert!(piece.is_some());

    let (piece_type, color) = piece.unwrap();
    assert_eq!(piece_type, PieceType::King);
    assert_eq!(color, Color::Red);

    // Remove the piece
    board.remove_piece_at(4, 0);
    assert!(board.get_piece_at(4, 0).is_none());
}
