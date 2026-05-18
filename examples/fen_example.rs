//! Example demonstrating FEN (Forsyth-Edwards Notation) functionality in cchess-rs

use cchess_rs::board::Board;

fn main() {
    println!("=== Chinese Chess FEN Example ===\n");

    // 1. Create a board from standard FEN
    let standard_fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";
    println!("1. Creating board from standard FEN:");
    println!("   FEN: {}", standard_fen);

    match Board::from_fen(standard_fen) {
        Ok(board) => {
            println!("   Successfully parsed FEN!");

            // Display some key pieces
            println!("   Red king at (4,0): {}", board.get_fen(4, 0));
            println!("   Black king at (4,9): {}", board.get_fen(4, 9));
            println!("   Red cannon at (1,2): {}", board.get_fen(1, 2));
            println!("   Black cannon at (1,7): {}", board.get_fen(1, 7));

            // Convert back to FEN
            let generated_fen = board.to_fen();
            println!("   Generated FEN: {}", generated_fen);
            println!("   FENs match: {}\n", standard_fen == generated_fen);
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // 2. Create an empty board and show its FEN
    println!("2. Creating empty board:");
    let mut empty_board = Board::default();
    empty_board.clear();

    let empty_fen = empty_board.to_fen();
    println!("   Empty board FEN: {}", empty_fen);
    println!("   Expected: 9/9/9/9/9/9/9/9/9/9");
    println!("   Match: {}\n", empty_fen == "9/9/9/9/9/9/9/9/9/9");

    // 3. Create a custom position
    println!("3. Creating custom position:");
    let custom_fen = "4k4/9/9/9/9/9/9/9/9/4K4"; // Just kings in the center
    match Board::from_fen(custom_fen) {
        Ok(board) => {
            println!("   Custom FEN: {}", custom_fen);
            println!("   Red king at (4,0): {}", board.get_fen(4, 0)); // Should be 'k'
            println!("   Black king at (4,9): {}", board.get_fen(4, 9)); // Should be 'K'

            // Verify all other squares are empty
            let mut empty_count = 0;
            for row in 0..10 {
                for col in 0..9 {
                    if board.get_fen(col, row) == '.' {
                        empty_count += 1;
                    }
                }
            }
            println!("   Empty squares: {}/90", empty_count);
            println!("   All other squares empty: {}\n", empty_count == 88);
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // 4. Test FEN with full game information
    println!("4. Parsing FEN with full game info:");
    let full_fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
    println!("   Full FEN: {}", full_fen);

    match Board::from_fen(full_fen) {
        Ok(board) => {
            println!("   Successfully parsed (ignored turn, move count, etc.)");
            println!("   Board FEN part: {}", board.to_fen());
            println!("   Note: Only board position is parsed, other info is ignored\n");
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // 5. Round-trip test
    println!("5. Round-trip test (FEN -> Board -> FEN):");
    let test_fens = [
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR",
        "9/9/9/9/4k4/9/9/9/9/4K4",
        "r1ba1abnr/9/1cn4c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR",
    ];

    for (i, fen) in test_fens.iter().enumerate() {
        match Board::from_fen(fen) {
            Ok(board) => {
                let round_trip_fen = board.to_fen();
                let success = fen == &round_trip_fen;
                println!("   Test {}: {}", i + 1, if success { "✓" } else { "✗" });
                if !success {
                    println!("     Original: {}", fen);
                    println!("     Round-trip: {}", round_trip_fen);
                }
            }
            Err(e) => println!("   Test {}: Error - {}", i + 1, e),
        }
    }

    println!("\n=== FEN Example Complete ===");
}
