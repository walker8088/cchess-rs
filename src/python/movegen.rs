//! Move generation and attack matrix functions for Python bindings

use pyo3::prelude::*;

use crate::pieces::Side;

use super::board::PyBoard;
use super::enums::{PyPieceType, PySide};

/// Generate all legal moves for the current position
#[pyfunction]
pub fn generate_legal_moves(board: &PyBoard) -> Vec<(usize, usize, usize, usize)> {
    let side = if board.inner.to_fen().split_whitespace().nth(1) == Some("w") {
        Side::Red
    } else {
        Side::Black
    };

    let mut moves = Vec::new();
    for row in 0..10 {
        for col in 0..9 {
            if let Some((piece_type, piece_side)) = board.inner.get_piece_at(col, row) {
                if piece_side == side {
                    let piece_moves = crate::move_gen::generate_piece_moves(
                        &board.inner,
                        piece_type,
                        piece_side,
                        col,
                        row,
                    );
                    for m in piece_moves {
                        moves.push((m.from_col, m.from_row, m.to_col, m.to_row));
                    }
                }
            }
        }
    }
    moves
}

/// Generate attack matrix for a side
#[pyfunction]
pub fn generate_attack_matrix(
    board: &PyBoard,
    side: PySide,
) -> Vec<Vec<Vec<(usize, usize, PyPieceType, PySide)>>> {
    let attacks = crate::attack_matrix::generate_attack_matrix(&board.inner, side.into());
    attacks
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|attackers| {
                    attackers
                        .into_iter()
                        .map(|(col, row, pt, s)| (col, row, PyPieceType::from(pt), PySide::from(s)))
                        .collect()
                })
                .collect()
        })
        .collect()
}

/// Check if a position is attacked by a side
#[pyfunction]
pub fn is_position_attacked(board: &PyBoard, col: usize, row: usize, side: PySide) -> bool {
    crate::attack_matrix::is_position_attacked(&board.inner, (col, row), side.into())
}

/// Check if a king is in check
#[pyfunction]
pub fn is_king_in_check(board: &PyBoard, side: PySide) -> bool {
    crate::attack_matrix::is_king_in_check(&board.inner, side.into())
}
