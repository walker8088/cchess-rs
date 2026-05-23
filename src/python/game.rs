//! PyGame and PyGameMetadata - Python wrappers for Game and GameMetadata

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::game::{Game, GameMetadata};

use super::board::PyBoard;
use super::enums::PySide;
use super::r#move::PyMoveNode;

// ============================================================================
// GameMetadata Wrapper
// ============================================================================

#[pyclass(name = "GameMetadata")]
#[derive(Clone)]
pub struct PyGameMetadata {
    inner: GameMetadata,
}

#[pymethods]
impl PyGameMetadata {
    #[new]
    fn new() -> Self {
        PyGameMetadata {
            inner: GameMetadata::default(),
        }
    }

    #[getter]
    fn title(&self) -> Option<String> {
        self.inner.title.clone()
    }

    #[setter(title)]
    fn set_title(&mut self, title: Option<String>) {
        self.inner.title = title;
    }

    #[getter]
    fn red_player(&self) -> Option<String> {
        self.inner.red_player.clone()
    }

    #[setter(red_player)]
    fn set_red_player(&mut self, red_player: Option<String>) {
        self.inner.red_player = red_player;
    }

    #[getter]
    fn black_player(&self) -> Option<String> {
        self.inner.black_player.clone()
    }

    #[setter(black_player)]
    fn set_black_player(&mut self, black_player: Option<String>) {
        self.inner.black_player = black_player;
    }

    #[getter]
    fn event(&self) -> Option<String> {
        self.inner.event.clone()
    }

    #[setter(event)]
    fn set_event(&mut self, event: Option<String>) {
        self.inner.event = event;
    }

    #[getter]
    fn date(&self) -> Option<String> {
        self.inner.date.clone()
    }

    #[setter(date)]
    fn set_date(&mut self, date: Option<String>) {
        self.inner.date = date;
    }

    #[getter]
    fn result(&self) -> Option<String> {
        self.inner.result.clone()
    }

    #[setter(result)]
    fn set_result(&mut self, result: Option<String>) {
        self.inner.result = result;
    }

    #[getter]
    fn source(&self) -> Option<String> {
        self.inner.source.clone()
    }

    #[setter(source)]
    fn set_source(&mut self, source: Option<String>) {
        self.inner.source = source;
    }

    #[getter]
    fn branch_count(&self) -> u32 {
        self.inner.branch_count
    }
}

// ============================================================================
// Game Wrapper
// ============================================================================

#[pyclass(name = "Game")]
pub struct PyGame {
    pub inner: Game,
}

#[pymethods]
impl PyGame {
    #[new]
    fn new() -> Self {
        PyGame { inner: Game::new() }
    }

    /// Create a game from a custom starting board
    #[staticmethod]
    fn from_board(board: &PyBoard) -> Self {
        PyGame {
            inner: Game::from_board(board.inner.clone()),
        }
    }

    /// Get the current board state
    fn get_board(&self) -> PyBoard {
        PyBoard {
            inner: self.inner.board.clone(),
        }
    }

    /// Get whose turn it is
    fn current_turn(&self) -> PySide {
        PySide::from(self.inner.current_turn)
    }

    /// Check if the game is over
    fn is_game_over(&self) -> bool {
        self.inner.is_game_over
    }

    /// Get the winner of the game (if game is over)
    fn winner(&self) -> Option<PySide> {
        self.inner.winner.map(PySide::from)
    }

    /// Get game metadata
    fn metadata(&self) -> PyGameMetadata {
        PyGameMetadata {
            inner: self.inner.metadata.clone(),
        }
    }

    /// Set game metadata
    fn set_metadata(&mut self, metadata: &PyGameMetadata) {
        self.inner.metadata = metadata.inner.clone();
    }

    /// Make a move on the main line
    fn make_move(
        &mut self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> PyResult<()> {
        self.inner
            .make_move((from_col, from_row), (to_col, to_row))
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Make a move as a variation
    fn make_variation(
        &mut self,
        parent_ply: u32,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> PyResult<()> {
        self.inner
            .make_variation(parent_ply, (from_col, from_row), (to_col, to_row))
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Navigate to a specific move in the tree
    fn navigate_to_move(&mut self, ply: u32) -> PyResult<()> {
        self.inner
            .navigate_to_move(ply)
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Get the main line moves
    fn get_main_line(&self) -> Vec<PyMoveNode> {
        self.inner
            .root_moves
            .iter()
            .flat_map(|node| {
                node.get_main_line()
                    .into_iter()
                    .map(|n| PyMoveNode { inner: n.clone() })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Get string representation of the move tree
    fn get_move_tree_string(&self) -> String {
        self.inner.get_move_tree_string()
    }

    /// Get total number of moves in main line
    fn total_moves(&self) -> usize {
        self.inner.root_moves.iter().map(|n| n.count_moves()).sum()
    }

    /// Get total number of variations
    fn total_variations(&self) -> u32 {
        self.inner
            .root_moves
            .iter()
            .map(|n| n.count_variations())
            .sum()
    }

    /// Convert game to PGN format
    fn to_pgn(&self) -> String {
        self.inner.to_pgn()
    }

    /// Check if the current side is in check
    fn is_in_check(&self) -> bool {
        self.inner.board.is_in_check(self.inner.current_turn)
    }

    fn __str__(&self) -> String {
        self.inner.to_pgn()
    }

    fn __repr__(&self) -> String {
        let total: usize = self.inner.root_moves.iter().map(|n| n.count_moves()).sum();
        format!("Game(moves={}, turn={:?})", total, self.inner.current_turn)
    }
}
