use cchess_rs::engine::{Engine, Protocol};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for protocol mode command-line arguments
    let protocol = if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "uci" => Protocol::UCI,
            "ucci" => Protocol::UCCI,
            "--uci" => Protocol::UCI,
            "--ucci" => Protocol::UCCI,
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Usage: cchess-rs [uci|ucci]");
                eprintln!("  uci   - Run as UCI protocol engine");
                eprintln!("  ucci  - Run as UCCI protocol engine");
                eprintln!("  (no args) - Run in interactive mode");
                return;
            }
        }
    } else {
        // Default: interactive CLI mode
        println!("Welcome to Chinese Chess (Xiangqi) in Rust!");
        println!("cchess-rs version: {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Usage:");
        println!("  cchess-rs uci   - Run as UCI protocol engine");
        println!("  cchess-rs ucci  - Run as UCCI protocol engine");
        return;
    };

    // Run engine in protocol mode
    let mut engine = Engine::new(protocol);
    if let Err(e) = engine.run() {
        eprintln!("Engine error: {}", e);
        std::process::exit(1);
    }
}
