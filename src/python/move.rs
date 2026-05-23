//! PyMoveNode - Python wrapper for the MoveNode type

use pyo3::prelude::*;

use crate::game::MoveNode;

use super::board::PyBoard;
use super::enums::PySide;

#[pyclass(name = "MoveNode")]
#[derive(Clone)]
pub struct PyMoveNode {
    pub inner: MoveNode,
}

#[pymethods]
impl PyMoveNode {
    #[new]
    fn new(
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        uci_notation: String,
        board_after: &PyBoard,
        next_turn: PySide,
        move_number: u32,
    ) -> Self {
        PyMoveNode {
            inner: MoveNode::new(
                (from_col, from_row),
                (to_col, to_row),
                uci_notation,
                board_after.inner.clone(),
                next_turn.into(),
                move_number,
            ),
        }
    }

    #[getter]
    fn from_col(&self) -> usize {
        self.inner.from.0
    }

    #[getter]
    fn from_row(&self) -> usize {
        self.inner.from.1
    }

    #[getter]
    fn to_col(&self) -> usize {
        self.inner.to.0
    }

    #[getter]
    fn to_row(&self) -> usize {
        self.inner.to.1
    }

    #[getter]
    fn uci_notation(&self) -> &str {
        &self.inner.uci_notation
    }

    #[getter]
    fn annotation(&self) -> Option<String> {
        self.inner.annotation.clone()
    }

    #[setter]
    fn set_annotation(&mut self, annotation: Option<String>) {
        self.inner.annotation = annotation;
    }

    #[getter]
    fn next_turn(&self) -> PySide {
        PySide::from(self.inner.next_turn)
    }

    #[getter]
    fn move_number(&self) -> u32 {
        self.inner.move_number
    }

    #[getter]
    fn board_after(&self) -> PyBoard {
        PyBoard {
            inner: self.inner.board_after.clone(),
        }
    }

    /// Get main line moves
    fn get_main_line(&self) -> Vec<PyMoveNode> {
        self.inner
            .get_main_line()
            .into_iter()
            .map(|node| PyMoveNode {
                inner: node.clone(),
            })
            .collect()
    }

    /// Count moves in main line
    fn count_moves(&self) -> usize {
        self.inner.count_moves()
    }

    /// Count all variations recursively
    fn count_variations(&self) -> u32 {
        self.inner.count_variations()
    }

    fn __str__(&self) -> String {
        format!(
            "{}. {} {}->{}",
            self.inner.move_number, self.inner.uci_notation, self.inner.from.0, self.inner.from.1
        )
    }
}
