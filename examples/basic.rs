//! Basic example of using cchess-rs

use cchess_rs::game::Game;

fn main() {
    println!("=== Chinese Chess (Xiangqi) Example ===");

    // Create a new game
    let game = Game::new();
    println!("Game created!");
    println!("{}", game.display());

    // Try to make a move (this will fail since Board::new() returns unimplemented)
    // Uncomment when Board::new() is implemented
    // match game.make_move((0, 0), (1, 1)) {
    //     Ok(_) => println!("Move successful!"),
    //     Err(e) => println!("Move failed: {}", e),
    // }

    // Display game state again
    println!("\nAfter move attempt:");
    println!("{}", game.display());

    println!("\n=== Example Complete ===");
}
