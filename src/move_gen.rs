/// Move generation module for Chinese Chess
use crate::board::Board;
use crate::pieces::{PieceType, Side};

/// Represents a move
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from_col: usize,
    pub from_row: usize,
    pub to_col: usize,
    pub to_row: usize,
    pub captured: Option<char>, // FEN character of captured piece
}

impl Move {
    /// Create a new move
    pub fn new(from_col: usize, from_row: usize, to_col: usize, to_row: usize) -> Self {
        Move {
            from_col,
            from_row,
            to_col,
            to_row,
            captured: None,
        }
    }

    /// Create a new move with capture
    pub fn with_capture(
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        captured: char, // FEN character
    ) -> Self {
        Move {
            from_col,
            from_row,
            to_col,
            to_row,
            captured: Some(captured),
        }
    }
}

/// Generate all legal moves for a given side
pub fn generate_moves(board: &Board, side: Side) -> Vec<Move> {
    let mut moves = Vec::new();

    for row in 0..10 {
        for col in 0..9 {
            if board.is_color_at(col, row, side) {
                if let Some(piece_type) = board.get_piece_type(col, row) {
                    let piece_moves = generate_piece_moves(board, piece_type, side, col, row);
                    moves.extend(piece_moves);
                }
            }
        }
    }

    moves
}

/// Generate moves for a specific piece
pub fn generate_piece_moves(
    board: &Board,
    piece_type: PieceType,
    side: Side,
    col: usize,
    row: usize,
) -> Vec<Move> {
    match piece_type {
        PieceType::King => generate_king_moves(board, side, col, row),
        PieceType::Advisor => generate_advisor_moves(board, side, col, row),
        PieceType::Elephant => generate_elephant_moves(board, side, col, row),
        PieceType::Knight => generate_knight_moves(board, side, col, row),
        PieceType::Rook => generate_rook_moves(board, side, col, row),
        PieceType::Cannon => generate_cannon_moves(board, side, col, row),
        PieceType::Pawn => generate_pawn_moves(board, side, col, row),
    }
}

/// Generate moves for the King (将/帅)
fn generate_king_moves(board: &Board, side: Side, col: usize, row: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // Up, Down, Right, Left

    for (dc, dr) in directions.iter() {
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;

        // General must stay within the palace
        // Red (红方) palace: rows 0-2 (bottom)
        // Black (黑方) palace: rows 7-9 (top)
        if (3..=5).contains(&new_col)
            && ((side == Side::Red && (0..=2).contains(&new_row))
                || (side == Side::Black && (7..=9).contains(&new_row)))
        {
            if let Some(m) = try_move(board, side, col, row, new_col as usize, new_row as usize) {
                moves.push(m);
            }
        }
    }

    moves
}

/// Generate moves for the Advisor (士/仕)
fn generate_advisor_moves(board: &Board, side: Side, col: usize, row: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)]; // Diagonal

    for (dc, dr) in directions.iter() {
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;

        // Advisor must stay within the palace
        // Red (红方) palace: rows 0-2 (bottom)
        // Black (黑方) palace: rows 7-9 (top)
        if (3..=5).contains(&new_col)
            && ((side == Side::Red && (0..=2).contains(&new_row))
                || (side == Side::Black && (7..=9).contains(&new_row)))
        {
            if let Some(m) = try_move(board, side, col, row, new_col as usize, new_row as usize) {
                moves.push(m);
            }
        }
    }

    moves
}

/// Generate moves for the Elephant (象/相)
fn generate_elephant_moves(board: &Board, side: Side, col: usize, row: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(2, 2), (2, -2), (-2, 2), (-2, -2)]; // Diagonal 2 squares
    let blocks = [(1, 1), (1, -1), (-1, 1), (-1, -1)]; // Blocking points

    for i in 0..4 {
        let (dc, dr) = directions[i];
        let (bc, br) = blocks[i];
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;
        let block_col = col as isize + bc;
        let block_row = row as isize + br;

        // Elephant cannot cross the river
        // Red elephant: rows 0-4 (bottom half)
        // Black elephant: rows 5-9 (top half)
        if (0..=8).contains(&new_col) && (0..=9).contains(&new_row) {
            if side == Side::Red && new_row <= 4 {
                // Check if the blocking point is empty
                if board.is_empty_at(block_col as usize, block_row as usize) {
                    if let Some(m) =
                        try_move(board, side, col, row, new_col as usize, new_row as usize)
                    {
                        moves.push(m);
                    }
                }
            } else if side == Side::Black
                && new_row >= 5
                && board.is_empty_at(block_col as usize, block_row as usize)
            {
                if let Some(m) = try_move(board, side, col, row, new_col as usize, new_row as usize)
                {
                    moves.push(m);
                }
            }
        }
    }

    moves
}

/// Generate moves for the Knight (马/傌)
fn generate_knight_moves(board: &Board, side: Side, col: usize, row: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    // Horse moves: 2 in one direction, 1 perpendicular (L-shape)
    // With blocking considerations
    let moves_pattern = [
        (2, 1, 1, 0),    // Right 2, Up 1 - block at (1, 0)
        (2, -1, 1, 0),   // Right 2, Down 1 - block at (1, 0)
        (-2, 1, -1, 0),  // Left 2, Up 1 - block at (-1, 0)
        (-2, -1, -1, 0), // Left 2, Down 1 - block at (-1, 0)
        (1, 2, 0, 1),    // Up 2, Right 1 - block at (0, 1)
        (1, -2, 0, -1),  // Down 2, Right 1 - block at (0, -1)
        (-1, 2, 0, 1),   // Up 2, Left 1 - block at (0, 1)
        (-1, -2, 0, -1), // Down 2, Left 1 - block at (0, -1)
    ];

    for (dc, dr, bc, br) in moves_pattern.iter() {
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;
        let block_col = col as isize + bc;
        let block_row = row as isize + br;

        if (0..=8).contains(&new_col) && (0..=9).contains(&new_row) {
            // Check if the blocking point is empty
            if board.is_empty_at(block_col as usize, block_row as usize) {
                if let Some(m) = try_move(board, side, col, row, new_col as usize, new_row as usize)
                {
                    moves.push(m);
                }
            }
        }
    }

    moves
}

/// Generate moves for the Rook (车/俥)
fn generate_rook_moves(board: &Board, side: Side, col: usize, row: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // Up, Down, Right, Left

    for (dc, dr) in directions.iter() {
        let mut new_col = col as isize + dc;
        let mut new_row = row as isize + dr;

        while (0..=8).contains(&new_col) && (0..=9).contains(&new_row) {
            if let Some(m) = try_move(board, side, col, row, new_col as usize, new_row as usize) {
                let is_capture = m.captured.is_some();
                moves.push(m);
                if is_capture {
                    break; // Stop after capturing
                }
            } else {
                break; // Blocked by own piece
            }
            new_col += dc;
            new_row += dr;
        }
    }

    moves
}

/// Generate moves for the Cannon (炮/砲)
fn generate_cannon_moves(board: &Board, side: Side, col: usize, row: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // Up, Down, Right, Left

    for (dc, dr) in directions.iter() {
        let mut new_col = col as isize + dc;
        let mut new_row = row as isize + dr;
        let mut jumped = false;

        while (0..=8).contains(&new_col) && (0..=9).contains(&new_row) {
            if !jumped {
                if board.has_piece_at(new_col as usize, new_row as usize) {
                    // Found a piece to jump over
                    jumped = true;
                } else {
                    // Can move to empty square
                    if let Some(m) =
                        try_move(board, side, col, row, new_col as usize, new_row as usize)
                    {
                        moves.push(m);
                    }
                }
            } else {
                // After jumping, can only capture
                if board.has_piece_at(new_col as usize, new_row as usize) {
                    let target_fen = board.get_fen(new_col as usize, new_row as usize);
                    if let Some(target_side) = Side::from_fen(target_fen) {
                        if target_side != side {
                            if let Some(m) =
                                try_move(board, side, col, row, new_col as usize, new_row as usize)
                            {
                                moves.push(m);
                            }
                        }
                    }
                    break; // Stop after capturing or finding own piece
                }
            }
            new_col += dc;
            new_row += dr;
        }
    }

    moves
}

/// Generate moves for the Pawn (卒/兵)
fn generate_pawn_moves(board: &Board, side: Side, col: usize, row: usize) -> Vec<Move> {
    let mut moves = Vec::new();

    // Pawn moves forward before crossing river, forward/sideways after
    // Red (红方) starts at rows 0-3, crosses river at row >= 5
    // Black (黑方) starts at rows 6-9, crosses river at row <= 4
    // Red moves downward (increasing row), Black moves upward (decreasing row)
    let forward = if side == Side::Red { 1 } else { -1 };
    let crossed_river = if side == Side::Red {
        row >= 5
    } else {
        row <= 4
    };

    // Forward move
    let new_col = col as isize;
    let new_row = row as isize + forward;
    if (0..=9).contains(&new_row) {
        if let Some(m) = try_move(board, side, col, row, new_col as usize, new_row as usize) {
            moves.push(m);
        }
    }

    // After crossing river, can also move sideways
    if crossed_river {
        for dc in [-1, 1].iter() {
            let new_col = col as isize + dc;
            let new_row = row as isize;
            if (0..=8).contains(&new_col) {
                if let Some(m) = try_move(board, side, col, row, new_col as usize, new_row as usize)
                {
                    moves.push(m);
                }
            }
        }
    }

    moves
}

/// Try to make a move, returns Some(Move) if valid, None otherwise
pub fn try_move(
    board: &Board,
    side: Side,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> Option<Move> {
    let target_fen = board.get_fen(to_col, to_row);

    if target_fen != '.' {
        // There is a piece at the target square
        if let Some(target_side) = Side::from_fen(target_fen) {
            if target_side == side {
                return None; // Cannot capture own piece
            }
            // Capture enemy piece
            return Some(Move::with_capture(
                from_col, from_row, to_col, to_row, target_fen,
            ));
        }
        // Invalid FEN character (shouldn't happen)
        return None;
    }

    // Move to empty square
    Some(Move::new(from_col, from_row, to_col, to_row))
}
