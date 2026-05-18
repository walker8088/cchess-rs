/// Game module for Chinese Chess

use crate::board::Board;
use crate::pieces::Color;

/// Represents the game state
pub struct Game {
    /// The current board state
    pub board: Board,
    /// The side whose turn it is
    pub current_turn: Color,
    /// Whether the game is over
    pub is_game_over: bool,
    /// The winner of the game (if game is over)
    pub winner: Option<Color>,
    /// Move history
    pub move_history: Vec<String>,
}

impl Game {
    /// Create a new game with initial position
    pub fn new() -> Self {
        Game {
            board: Board::new(),
            current_turn: Color::Red, // Red always moves first in Chinese Chess
            is_game_over: false,
            winner: None,
            move_history: Vec::new(),
        }
    }

    /// Make a move in algebraic notation
    pub fn make_move(&mut self, from: (usize, usize), to: (usize, usize)) -> Result<(), String> {
        if self.is_game_over {
            return Err("Game is already over".to_string());
        }

        // Validate and execute the move
        if self.board.make_move(from, to) {
            // Record the move
            let move_str = format!("({},{}) -> ({},{})", from.0, from.1, to.0, to.1);
            self.move_history.push(move_str);

            // Switch turns
            self.current_turn = self.current_turn.opposite();

            // Check for game over conditions
            self.check_game_over();

            Ok(())
        } else {
            Err("Invalid move".to_string())
        }
    }

    /// Check if the game is over
    fn check_game_over(&mut self) {
        // TODO: Implement checkmate and stalemate detection
        // For now, just check if the general is captured
        let red_general_exists = self.find_general(Color::Red).is_some();
        let black_general_exists = self.find_general(Color::Black).is_some();

        if !red_general_exists {
            self.is_game_over = true;
            self.winner = Some(Color::Black);
        } else if !black_general_exists {
            self.is_game_over = true;
            self.winner = Some(Color::Red);
        }
    }

    /// Find the position of a general
    fn find_general(&self, color: Color) -> Option<(usize, usize)> {
        use crate::pieces::PieceType;

        for rank in 0..10 {
            for file in 0..9 {
                if let Some(piece) = self.board.get_piece(file, rank) {
                    if piece.piece_type == PieceType::General && piece.color == color {
                        return Some((file, rank));
                    }
                }
            }
        }
        None
    }

    /// Get the current game state as a string
    pub fn display(&self) -> String {
        format!(
            "Current turn: {:?}\nGame over: {}\nWinner: {:?}",
            self.current_turn, self.is_game_over, self.winner
        )
    }
}
