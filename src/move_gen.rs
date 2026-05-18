/// Move generation module for Chinese Chess

use crate::board::Board;
use crate::pieces::{Color, Piece, PieceType};

/// Represents a move
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from_file: usize,
    pub from_rank: usize,
    pub to_file: usize,
    pub to_rank: usize,
    pub captured: Option<Piece>,
}

impl Move {
    /// Create a new move
    pub fn new(from_file: usize, from_rank: usize, to_file: usize, to_rank: usize) -> Self {
        Move {
            from_file,
            from_rank,
            to_file,
            to_rank,
            captured: None,
        }
    }

    /// Create a new move with capture
    pub fn with_capture(
        from_file: usize,
        from_rank: usize,
        to_file: usize,
        to_rank: usize,
        captured: Piece,
    ) -> Self {
        Move {
            from_file,
            from_rank,
            to_file,
            to_rank,
            captured: Some(captured),
        }
    }
}

/// Generate all legal moves for a given color
pub fn generate_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();

    for rank in 0..10 {
        for file in 0..9 {
            if let Some(piece) = board.get_piece(file, rank) {
                if piece.color == color {
                    let piece_moves = generate_piece_moves(board, piece, file, rank);
                    moves.extend(piece_moves);
                }
            }
        }
    }

    moves
}

/// Generate moves for a specific piece
fn generate_piece_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    match piece.piece_type {
        PieceType::General => generate_general_moves(board, piece, file, rank),
        PieceType::Advisor => generate_advisor_moves(board, piece, file, rank),
        PieceType::Elephant => generate_elephant_moves(board, piece, file, rank),
        PieceType::Horse => generate_horse_moves(board, piece, file, rank),
        PieceType::Chariot => generate_chariot_moves(board, piece, file, rank),
        PieceType::Cannon => generate_cannon_moves(board, piece, file, rank),
        PieceType::Soldier => generate_soldier_moves(board, piece, file, rank),
    }
}

/// Generate moves for the General (将/帅)
fn generate_general_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // Up, Down, Right, Left

    for (df, dr) in directions.iter() {
        let new_file = file as isize + df;
        let new_rank = rank as isize + dr;

        // General must stay within the palace
        if new_file >= 3 && new_file <= 5 {
            if piece.color == Color::Red && new_rank >= 0 && new_rank <= 2 {
                if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                    moves.push(m);
                }
            } else if piece.color == Color::Black && new_rank >= 7 && new_rank <= 9 {
                if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                    moves.push(m);
                }
            }
        }
    }

    moves
}

/// Generate moves for the Advisor (士/仕)
fn generate_advisor_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)]; // Diagonal

    for (df, dr) in directions.iter() {
        let new_file = file as isize + df;
        let new_rank = rank as isize + dr;

        // Advisor must stay within the palace
        if new_file >= 3 && new_file <= 5 {
            if piece.color == Color::Red && new_rank >= 0 && new_rank <= 2 {
                if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                    moves.push(m);
                }
            } else if piece.color == Color::Black && new_rank >= 7 && new_rank <= 9 {
                if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                    moves.push(m);
                }
            }
        }
    }

    moves
}

/// Generate moves for the Elephant (象/相)
fn generate_elephant_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(2, 2), (2, -2), (-2, 2), (-2, -2)]; // Diagonal 2 squares
    let blocks = [(1, 1), (1, -1), (-1, 1), (-1, -1)]; // Blocking points

    for i in 0..4 {
        let (df, dr) = directions[i];
        let (bf, br) = blocks[i];
        let new_file = file as isize + df;
        let new_rank = rank as isize + dr;
        let block_file = file as isize + bf;
        let block_rank = rank as isize + br;

        // Elephant cannot cross the river
        // Red elephant: ranks 0-4, Black elephant: ranks 5-9
        if new_file >= 0 && new_file <= 8 && new_rank >= 0 && new_rank <= 9 {
            if piece.color == Color::Red && new_rank <= 4 {
                // Check if the blocking point is empty
                if board.get_piece(block_file as usize, block_rank as usize).is_none() {
                    if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                        moves.push(m);
                    }
                }
            } else if piece.color == Color::Black && new_rank >= 5 {
                if board.get_piece(block_file as usize, block_rank as usize).is_none() {
                    if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                        moves.push(m);
                    }
                }
            }
        }
    }

    moves
}

/// Generate moves for the Horse (马/傌)
fn generate_horse_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    // Horse moves: 2 in one direction, 1 perpendicular (L-shape)
    // With blocking considerations
    let moves_pattern = [
        (2, 1, 1, 0),   // Right 2, Up 1 - block at (1, 0)
        (2, -1, 1, 0),  // Right 2, Down 1 - block at (1, 0)
        (-2, 1, -1, 0), // Left 2, Up 1 - block at (-1, 0)
        (-2, -1, -1, 0), // Left 2, Down 1 - block at (-1, 0)
        (1, 2, 0, 1),   // Up 2, Right 1 - block at (0, 1)
        (1, -2, 0, -1), // Down 2, Right 1 - block at (0, -1)
        (-1, 2, 0, 1),  // Up 2, Left 1 - block at (0, 1)
        (-1, -2, 0, -1), // Down 2, Left 1 - block at (0, -1)
    ];

    for (df, dr, bf, br) in moves_pattern.iter() {
        let new_file = file as isize + df;
        let new_rank = rank as isize + dr;
        let block_file = file as isize + bf;
        let block_rank = rank as isize + br;

        if new_file >= 0 && new_file <= 8 && new_rank >= 0 && new_rank <= 9 {
            // Check if the blocking point is empty
            if board.get_piece(block_file as usize, block_rank as usize).is_none() {
                if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                    moves.push(m);
                }
            }
        }
    }

    moves
}

/// Generate moves for the Chariot (车/俥)
fn generate_chariot_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // Up, Down, Right, Left

    for (df, dr) in directions.iter() {
        let mut new_file = file as isize + df;
        let mut new_rank = rank as isize + dr;

        while new_file >= 0 && new_file <= 8 && new_rank >= 0 && new_rank <= 9 {
            if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                let is_capture = m.captured.is_some();
                moves.push(m);
                if is_capture {
                    break; // Stop after capturing
                }
            } else {
                break; // Blocked by own piece
            }
            new_file += df;
            new_rank += dr;
        }
    }

    moves
}

/// Generate moves for the Cannon (炮/砲)
fn generate_cannon_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // Up, Down, Right, Left

    for (df, dr) in directions.iter() {
        let mut new_file = file as isize + df;
        let mut new_rank = rank as isize + dr;
        let mut jumped = false;

        while new_file >= 0 && new_file <= 8 && new_rank >= 0 && new_rank <= 9 {
            if !jumped {
                if let Some(_target) = board.get_piece(new_file as usize, new_rank as usize) {
                    // Found a piece to jump over
                    jumped = true;
                } else {
                    // Can move to empty square
                    if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                        moves.push(m);
                    }
                }
            } else {
                // After jumping, can only capture
                if let Some(target) = board.get_piece(new_file as usize, new_rank as usize) {
                    if target.color != piece.color {
                        if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                            moves.push(m);
                        }
                    }
                    break; // Stop after capturing or finding own piece
                }
            }
            new_file += df;
            new_rank += dr;
        }
    }

    moves
}

/// Generate moves for the Soldier (卒/兵)
fn generate_soldier_moves(board: &Board, piece: &Piece, file: usize, rank: usize) -> Vec<Move> {
    let mut moves = Vec::new();

    // Soldier moves forward before crossing river, forward/sideways after
    let forward = if piece.color == Color::Red { 1 } else { -1 };
    let crossed_river = if piece.color == Color::Red {
        rank >= 5
    } else {
        rank <= 4
    };

    // Forward move
    let new_file = file as isize;
    let new_rank = rank as isize + forward;
    if new_rank >= 0 && new_rank <= 9 {
        if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
            moves.push(m);
        }
    }

    // After crossing river, can also move sideways
    if crossed_river {
        for df in [-1, 1].iter() {
            let new_file = file as isize + df;
            let new_rank = rank as isize;
            if new_file >= 0 && new_file <= 8 {
                if let Some(m) = try_move(board, piece, file, rank, new_file as usize, new_rank as usize) {
                    moves.push(m);
                }
            }
        }
    }

    moves
}

/// Try to make a move, returns Some(Move) if valid, None otherwise
fn try_move(
    board: &Board,
    piece: &Piece,
    from_file: usize,
    from_rank: usize,
    to_file: usize,
    to_rank: usize,
) -> Option<Move> {
    if let Some(target) = board.get_piece(to_file, to_rank) {
        if target.color == piece.color {
            return None; // Cannot capture own piece
        }
        Some(Move::with_capture(from_file, from_rank, to_file, to_rank, *target))
    } else {
        Some(Move::new(from_file, from_rank, to_file, to_rank))
    }
}
