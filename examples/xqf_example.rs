//! Example of reading and writing XQF files
use cchess_rs::board::Board;
use cchess_rs::game::Game;
use cchess_rs::xqf::{board_from_xqf, board_to_xqf, XqfFile};

fn main() {
    println!("XQF File Format Example");
    println!("=======================\n");

    // Example 1: Create a new game and convert to XQF
    println!("1. Creating a new game and converting to XQF format:");
    let game = Game::new();

    match XqfFile::from_game(&game, "Test Match", "Red Player", "Black Player") {
        Ok(xqf_file) => {
            println!(
                "   Created XQF file with title: {}",
                xqf_file.game_info.title
            );
            println!("   Red player: {}", xqf_file.game_info.red_player);
            println!("   Black player: {}", xqf_file.game_info.black_player);
            println!("   Number of moves: {}", xqf_file.moves.len());
            println!("   Has initial board: {}", xqf_file.initial_board.is_some());
        }
        Err(e) => {
            println!("   Error creating XQF file: {:?}", e);
        }
    }

    println!();

    // Example 2: Test board to XQF conversion
    println!("2. Testing board to XQF conversion:");
    let board = Board::new();

    match board_to_xqf(&board) {
        Ok(xqf_data) => {
            println!("   Successfully converted board to XQF format");
            println!("   Data length: {} bytes", xqf_data.len());

            // Count non-zero entries (pieces)
            let piece_count = xqf_data.iter().filter(|&&b| b != 0).count();
            println!("   Pieces on board: {}", piece_count);

            // Show some specific pieces
            let red_king_pos = 4; // (4, 0)
            let black_king_pos = 9 * 9 + 4; // (4, 9)

            println!(
                "   Red king at position {}: code {}",
                red_king_pos, xqf_data[red_king_pos]
            );
            println!(
                "   Black king at position {}: code {}",
                black_king_pos, xqf_data[black_king_pos]
            );
        }
        Err(e) => {
            println!("   Error converting board: {}", e);
        }
    }

    println!();

    // Example 3: Test XQF to board conversion
    println!("3. Testing XQF to board conversion:");
    let mut test_data = [0u8; 90];

    // Place a red king at (4, 0)
    test_data[4] = 1;
    // Place a black rook at (0, 9)
    test_data[9 * 9 + 0] = 13;

    match board_from_xqf(&test_data) {
        Ok(board) => {
            println!("   Successfully created board from XQF data");

            // Check the pieces
            if let Some((piece_type, color)) = board.get_piece_at(4, 0) {
                println!("   Found {:?} ({:?}) at (4, 0)", piece_type, color);
            }

            if let Some((piece_type, color)) = board.get_piece_at(0, 9) {
                println!("   Found {:?} ({:?}) at (0, 9)", piece_type, color);
            }
        }
        Err(e) => {
            println!("   Error creating board: {}", e);
        }
    }

    println!();
    println!("Example completed successfully!");
}
