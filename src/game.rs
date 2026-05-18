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
        // First check if kings exist
        let red_king_pos = self.find_king(Color::Red);
        let black_king_pos = self.find_king(Color::Black);

        if red_king_pos.is_none() {
            self.is_game_over = true;
            self.winner = Some(Color::Black);
            return;
        }

        if black_king_pos.is_none() {
            self.is_game_over = true;
            self.winner = Some(Color::Red);
            return;
        }

        // Check if red king is in check
        let is_red_in_check = self.is_in_check(Color::Red);

        // Check if black king is in check
        let is_black_in_check = self.is_in_check(Color::Black);

        // Check for checkmate
        if is_red_in_check {
            // Check if red has any legal moves
            if !self.has_legal_moves(Color::Red) {
                self.is_game_over = true;
                self.winner = Some(Color::Black);
                return;
            }
        }

        if is_black_in_check {
            // Check if black has any legal moves
            if !self.has_legal_moves(Color::Black) {
                self.is_game_over = true;
                self.winner = Some(Color::Red);
                return;
            }
        }

        // Check for stalemate (no legal moves but not in check)
        if !is_red_in_check && !self.has_legal_moves(Color::Red) {
            self.is_game_over = true;
            self.winner = None; // Draw
            return;
        }

        if !is_black_in_check && !self.has_legal_moves(Color::Black) {
            self.is_game_over = true;
            self.winner = None; // Draw
            return;
        }
    }

    /// Check if a king is in check
    fn is_in_check(&self, color: Color) -> bool {
        // Find the king's position
        if let Some((king_col, king_row)) = self.find_king(color) {
            let opponent_color = color.opposite();

            // Check all opponent pieces to see if they can attack the king
            for row in 0..10 {
                for col in 0..9 {
                    if self.board.is_color_at(col, row, opponent_color) {
                        if let Some(piece_type) = self.board.get_piece_type(col, row) {
                            // Check if this piece can attack the king
                            if self.can_attack_king(piece_type, color, col, row, king_col, king_row)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if a piece can attack the king
    fn can_attack_king(
        &self,
        piece_type: crate::pieces::PieceType,
        defender_color: Color,
        attacker_col: usize,
        attacker_row: usize,
        king_col: usize,
        king_row: usize,
    ) -> bool {
        // For now, we'll use a simplified check
        // In a full implementation, we would need to check piece movement rules
        // and obstacles

        match piece_type {
            crate::pieces::PieceType::Rook => {
                // Rook can attack if on same row or column with no pieces in between
                (attacker_col == king_col || attacker_row == king_row)
                    && !self.board.has_pieces_between(
                        attacker_col,
                        attacker_row,
                        king_col,
                        king_row,
                    )
            }
            crate::pieces::PieceType::Cannon => {
                // Cannon can attack if on same row or column with exactly one piece in between
                if attacker_col == king_col || attacker_row == king_row {
                    self.board
                        .has_cannon_screen(attacker_col, attacker_row, king_col, king_row)
                } else {
                    false
                }
            }
            crate::pieces::PieceType::Knight => {
                // Knight can attack if in "L" shape
                let dx = (king_col as isize - attacker_col as isize).abs();
                let dy = (king_row as isize - attacker_row as isize).abs();

                (dx == 1 && dy == 2) || (dx == 2 && dy == 1)
            }
            crate::pieces::PieceType::Pawn => {
                // Pawn can attack adjacent squares after crossing river
                let dx = (king_col as isize - attacker_col as isize).abs();
                let dy = (king_row as isize - attacker_row as isize).abs();

                // Pawn moves differently based on color and whether it has crossed river
                let is_red = defender_color == Color::Black; // Attacker is opposite color
                let crossed_river = if is_red {
                    attacker_row >= 5
                } else {
                    attacker_row <= 4
                };

                if dx == 0 {
                    // Forward attack
                    let forward = if is_red { 1 } else { -1 };
                    attacker_row as isize + forward == king_row as isize
                } else {
                    // Sideways attack (only after crossing river)
                    crossed_river && dx == 1 && dy == 0
                }
            }
            // For other pieces, we need more complex logic
            crate::pieces::PieceType::King
            | crate::pieces::PieceType::Advisor
            | crate::pieces::PieceType::Elephant => {
                // These pieces have limited movement range
                // They can only attack if the king is within their movement range
                // This is a simplified check
                let dx = (king_col as isize - attacker_col as isize).abs();
                let dy = (king_row as isize - attacker_row as isize).abs();

                match piece_type {
                    crate::pieces::PieceType::King => dx + dy == 1, // Adjacent squares
                    crate::pieces::PieceType::Advisor => dx == 1 && dy == 1, // Diagonal
                    crate::pieces::PieceType::Elephant => dx == 2 && dy == 2, // Two squares diagonal
                    _ => false,
                }
            }
        }
    }

    /// Check if a color has any legal moves
    fn has_legal_moves(&self, color: Color) -> bool {
        use crate::move_gen::generate_moves;

        // Generate all legal moves for this color
        let moves = generate_moves(&self.board, color);

        // Check if any move is legal
        for mv in moves {
            // Create a copy of the board to test the move
            let mut test_board = self.board.copy();
            if test_board.make_move((mv.from_col, mv.from_row), (mv.to_col, mv.to_row)) {
                // Check if this move would leave the general in check
                let test_game = Game {
                    board: test_board,
                    current_turn: color.opposite(),
                    is_game_over: false,
                    winner: None,
                    move_history: Vec::new(),
                };

                if !test_game.is_in_check(color) {
                    return true; // Found at least one legal move
                }
            }
        }

        false // No legal moves found
    }

    /// Find the position of a king
    fn find_king(&self, color: Color) -> Option<(usize, usize)> {
        use crate::pieces::PieceType;

        for row in 0..10 {
            for col in 0..9 {
                if self.board.is_color_at(col, row, color) {
                    if let Some(piece_type) = self.board.get_piece_type(col, row) {
                        if piece_type == PieceType::King {
                            return Some((col, row));
                        }
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
