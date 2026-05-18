/// Pieces module for Chinese Chess

/// Piece types in Chinese Chess
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    General,    // 将/帅
    Advisor,    // 士/仕
    Elephant,   // 象/相
    Horse,      // 马/傌
    Chariot,    // 车/俥
    Cannon,     // 炮/砲
    Soldier,    // 卒/兵
}

/// Colors in Chinese Chess
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,    // 红方
    Black,  // 黑方
}

impl Color {
    /// Get the opposite color
    pub fn opposite(&self) -> Color {
        match self {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }
}

/// Represents a piece on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    /// Create a new piece
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece {
            piece_type,
            color,
        }
    }
}
