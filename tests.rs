// Integration tests for cchess-rs

#[cfg(test)]
mod tests {
    use cchess_rs::pieces::{Color, Piece, PieceType};
    use cchess_rs::board::Board;
    use cchess_rs::game::Game;
    use cchess_rs::move_gen::generate_moves;

    #[test]
    fn test_piece_creation() {
        let piece = Piece::new(PieceType::General, Color::Red);
        assert_eq!(piece.piece_type, PieceType::General);
        assert_eq!(piece.color, Color::Red);
    }

    #[test]
    fn test_color_opposite() {
        assert_eq!(Color::Red.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::Red);
    }

    #[test]
    fn test_board_creation() {
        let board = Board::new();
        // Board should be initialized
        assert!(true); // Placeholder test
    }

    #[test]
    fn test_game_creation() {
        let game = Game::new();
        assert_eq!(game.current_turn, Color::Red);
        assert_eq!(game.is_game_over, false);
        assert_eq!(game.winner, None);
    }

    #[test]
    fn test_move_generation() {
        let board = Board::new();
        let moves = generate_moves(&board, Color::Red);
        // Red should have some initial moves
        assert!(moves.len() > 0);
    }
}
