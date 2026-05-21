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

/// Sides in Chinese Chess (matches Python: SIDE_ANY=0, SIDE_RED=1, SIDE_BLACK=2)
/// Red = 红方 = uppercase FEN chars = "w" in FEN side-to-move
/// Black = 黑方 = lowercase FEN chars = "b" in FEN side-to-move
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Any,   // 任意边 (any side)
    Red,   // 红方 (uppercase FEN chars, "w" in FEN)
    Black, // 黑方 (lowercase FEN chars, "b" in FEN)
}

impl Side {
    /// Get the opposite side
    pub fn opposite(&self) -> Side {
        match self {
            Side::Any => Side::Any,
            Side::Red => Side::Black,
            Side::Black => Side::Red,
        }
    }

    /// Check if a FEN character belongs to this side
    pub fn matches_fen(&self, fen_char: char) -> bool {
        match self {
            Side::Any => fen_char != '.',
            Side::Red => fen_char.is_uppercase(),
            Side::Black => fen_char.is_lowercase() && fen_char != '.',
        }
    }

    /// Get side from FEN character
    pub fn from_fen(fen_char: char) -> Option<Side> {
        // First check if it's a valid FEN character
        if PieceType::from_fen(fen_char).is_none() && fen_char != '.' {
            return None;
        }

        if fen_char.is_lowercase() && fen_char != '.' {
            Some(Side::Black)
        } else if fen_char.is_uppercase() {
            Some(Side::Red)
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
