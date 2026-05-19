//! Pieces module for Chinese Chess
/// Piece types in Chinese Chess
/// Using international chess notation for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    King,     // 将/帅 (K/k in FEN)
    Advisor,  // 士/仕 (A/a in FEN)
    Elephant, // 象/相 (B/b in FEN) - 使用Bishop的'b'字符
    Knight,   // 马/傌 (N/n in FEN)
    Rook,     // 车/俥 (R/r in FEN)
    Cannon,   // 炮/砲 (C/c in FEN)
    Pawn,     // 兵/卒 (P/p in FEN)
}

/// Colors in Chinese Chess
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Any,   // 任意颜色
    Red,   // 红方
    Black, // 黑方
}

impl Color {
    /// Get the opposite color
    pub fn opposite(&self) -> Color {
        match self {
            Color::Any => Color::Any,
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }

    /// Check if a FEN character belongs to this color
    pub fn matches_fen(&self, fen_char: char) -> bool {
        match self {
            Color::Any => fen_char != '.',
            Color::Red => fen_char.is_lowercase() && fen_char != '.',
            Color::Black => fen_char.is_uppercase(),
        }
    }

    /// Get color from FEN character
    pub fn from_fen(fen_char: char) -> Option<Color> {
        // First check if it's a valid FEN character
        if PieceType::from_fen(fen_char).is_none() && fen_char != '.' {
            return None;
        }

        if fen_char.is_lowercase() && fen_char != '.' {
            Some(Color::Red)
        } else if fen_char.is_uppercase() {
            Some(Color::Black)
        } else {
            None
        }
    }
}

impl PieceType {
    /// Get piece type from FEN character
    pub fn from_fen(fen_char: char) -> Option<PieceType> {
        match fen_char.to_ascii_lowercase() {
            'k' => Some(PieceType::King),
            'a' => Some(PieceType::Advisor),
            'b' => Some(PieceType::Elephant),
            'n' => Some(PieceType::Knight),
            'r' => Some(PieceType::Rook),
            'c' => Some(PieceType::Cannon),
            'p' => Some(PieceType::Pawn),
            _ => None,
        }
    }

    /// Convert piece type to FEN base character (lowercase for Red)
    pub fn to_fen_base(&self) -> char {
        match self {
            PieceType::King => 'k',
            PieceType::Advisor => 'a',
            PieceType::Elephant => 'b',
            PieceType::Knight => 'n',
            PieceType::Rook => 'r',
            PieceType::Cannon => 'c',
            PieceType::Pawn => 'p',
        }
    }
}
