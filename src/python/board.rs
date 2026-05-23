//! PyBoard - Python wrapper for the Board type

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::board::Board;

use super::enums::{PyMoveFormat, PyPieceType, PySide};

#[pyclass(name = "Board")]
#[derive(Clone)]
pub struct PyBoard {
    pub inner: Board,
}

#[pymethods]
impl PyBoard {
    #[new]
    fn new() -> Self {
        PyBoard {
            inner: Board::new(),
        }
    }

    /// Set up the board with the standard Chinese Chess initial position
    fn initial_position(&mut self) {
        self.inner.initial_position();
    }

    /// Create a board from a FEN string
    #[staticmethod]
    fn from_fen(fen: &str) -> PyResult<Self> {
        Board::from_fen(fen)
            .map(|board| PyBoard { inner: board })
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Convert the board to a FEN string (board position only)
    fn to_fen(&self) -> String {
        self.inner.to_fen()
    }

    /// Convert the board to a full FEN string (includes side to move)
    fn to_full_fen(&self, side_to_move: PySide) -> String {
        self.inner.to_full_fen(side_to_move.into())
    }

    /// Clear the board (remove all pieces)
    fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get piece at position, returns (piece_type, side) or None
    fn get_piece_at(&self, col: usize, row: usize) -> Option<(PyPieceType, PySide)> {
        self.inner
            .get_piece_at(col, row)
            .map(|(pt, side)| (PyPieceType::from(pt), PySide::from(side)))
    }

    /// Set piece at position
    fn set_piece_at(&mut self, col: usize, row: usize, piece_type: PyPieceType, side: PySide) {
        self.inner
            .set_piece_at(col, row, piece_type.into(), side.into());
    }

    /// Remove piece at position
    fn remove_piece_at(&mut self, col: usize, row: usize) {
        self.inner.remove_piece_at(col, row);
    }

    /// Check if a square is empty
    fn is_empty_at(&self, col: usize, row: usize) -> bool {
        self.inner.is_empty_at(col, row)
    }

    /// Check if a square contains a piece of the given side
    fn is_color_at(&self, col: usize, row: usize, side: PySide) -> bool {
        self.inner.is_color_at(col, row, side.into())
    }

    /// Get occupied color at position: Side enum or None for empty
    fn occupied(&self, col: usize, row: usize) -> Option<PySide> {
        self.inner.occupied(col, row).map(PySide::from)
    }

    /// Make a move, returns true if successful
    fn make_move(
        &mut self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> bool {
        self.inner.make_move((from_col, from_row), (to_col, to_row))
    }

    /// Check if a move is valid (basic rules check)
    fn is_valid_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> bool {
        self.inner
            .is_valid_move((from_col, from_row), (to_col, to_row))
    }

    /// Check if a move would result in check (将军)
    fn is_checking_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> bool {
        self.inner
            .is_checking_move((from_col, from_row), (to_col, to_row))
    }

    /// Check if the king of the given side is in check
    fn is_in_check(&self, side: PySide) -> bool {
        self.inner.is_in_check(side.into())
    }

    /// Check if the side is checkmated (in check and no legal moves)
    fn is_checkmate(&self, side: PySide) -> bool {
        self.inner.is_checkmate(side.into())
    }

    /// Generate all legal moves for the given side
    /// Returns list of ((from_col, from_row), (to_col, to_row))
    fn create_moves(&self, side: PySide) -> Vec<((usize, usize), (usize, usize))> {
        self.inner.create_moves(side.into())
    }

    /// Mirror the board horizontally (left-right flip)
    fn mirror(&self) -> Self {
        PyBoard {
            inner: self.inner.mirror(),
        }
    }

    /// Flip the board vertically + horizontal mirror (perspective transform)
    fn flip(&self) -> Self {
        PyBoard {
            inner: self.inner.flip(),
        }
    }

    /// Swap piece colors (red <-> black)
    fn swap_colors(&self) -> Self {
        PyBoard {
            inner: self.inner.swap_colors(),
        }
    }

    /// Check if the board is horizontally symmetric
    fn is_mirror(&self) -> bool {
        self.inner.is_mirror()
    }

    /// Find king position for given side
    fn get_king_pos(&self, side: PySide) -> Option<(usize, usize)> {
        self.inner.get_king_pos(side.into())
    }

    /// Get all positions of a specific piece character (e.g., 'R', 'r')
    fn get_fench_positions(&self, fen_char: &str) -> Vec<(usize, usize)> {
        let c = fen_char.chars().next().unwrap_or('.');
        self.inner.get_fench_positions(c)
    }

    /// Get all positions of pieces for a given color
    fn get_all_fench_positions(&self, color: Option<PySide>) -> Vec<(String, usize, usize)> {
        self.inner
            .get_all_fench_positions(color.map(|s| s.into()))
            .into_iter()
            .map(|(c, col, row)| (c.to_string(), col, row))
            .collect()
    }

    /// Count pieces between two positions on x-axis (same row, exclusive)
    fn count_x_line_in(&self, row: usize, from_col: usize, to_col: usize) -> usize {
        self.inner.count_x_line_in(row, from_col, to_col)
    }

    /// Count pieces between two positions on y-axis (same col, exclusive)
    fn count_y_line_in(&self, col: usize, from_row: usize, to_row: usize) -> usize {
        self.inner.count_y_line_in(col, from_row, to_row)
    }

    /// Get pieces on x-line between positions (exclusive)
    fn x_line_in(&self, row: usize, from_col: usize, to_col: usize) -> Vec<String> {
        self.inner
            .x_line_in(row, from_col, to_col)
            .into_iter()
            .map(|c| c.to_string())
            .collect()
    }

    /// Get pieces on y-line between positions (exclusive)
    fn y_line_in(&self, col: usize, from_row: usize, to_row: usize) -> Vec<String> {
        self.inner
            .y_line_in(col, from_row, to_row)
            .into_iter()
            .map(|c| c.to_string())
            .collect()
    }

    /// Detect which pieces moved between two boards
    fn detect_move_pieces(&self, other: &PyBoard) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
        self.inner.detect_move_pieces(&other.inner)
    }

    /// Create a move from the difference between two boards
    fn create_move_from_board(&self, other: &PyBoard) -> Option<((usize, usize), (usize, usize))> {
        self.inner.create_move_from_board(&other.inner)
    }

    /// Pretty print board as text view
    fn print_view(&self) -> Vec<String> {
        self.inner.print_view()
    }

    /// Make a move by ICCS notation (e.g., "e2e4")
    fn move_iccs(&mut self, iccs: &str) -> bool {
        self.inner.move_iccs(iccs)
    }

    /// Get board as 2D array of characters
    fn get_squares(&self) -> Vec<Vec<String>> {
        self.inner
            .squares
            .iter()
            .map(|row| row.iter().map(|c| c.to_string()).collect())
            .collect()
    }

    /// Copy the board
    fn copy_board(&self) -> Self {
        PyBoard {
            inner: self.inner.clone(),
        }
    }

    /// Generate move text for a move
    fn move_text(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        format: PyMoveFormat,
        traditional: bool,
    ) -> PyResult<String> {
        self.inner
            .move_text(
                (from_col, from_row),
                (to_col, to_row),
                format.into(),
                traditional,
            )
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Generate Chinese move notation for a move
    fn move_notation(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        format: PyMoveFormat,
    ) -> PyResult<String> {
        self.inner
            .move_text((from_col, from_row), (to_col, to_row), format.into(), false)
            .map_err(|e| PyValueError::new_err(e))
    }

    /// String representation of the board
    fn __str__(&self) -> String {
        self.inner.to_fen()
    }

    fn __repr__(&self) -> String {
        format!("Board(fen='{}')", self.inner.to_fen())
    }
}
