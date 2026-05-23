//! PyMoveNotation - Python wrapper for MoveNotation

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::move_notation::{ChineseLocale, MoveNotation, Qualifier};

use super::board::PyBoard;
use super::enums::{PyChineseLocale, PyPieceType, PySide};

#[pyclass(name = "MoveNotation")]
#[derive(Clone)]
pub struct PyMoveNotation {
    inner: MoveNotation,
}

#[pymethods]
impl PyMoveNotation {
    /// Create move notation from board positions
    #[staticmethod]
    fn from_board(
        board: &PyBoard,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> PyResult<Self> {
        MoveNotation::from_board_move(&board.inner, (from_col, from_row), (to_col, to_row))
            .map(|notation| PyMoveNotation { inner: notation })
            .map_err(|e| PyValueError::new_err(e))
    }

    #[getter]
    fn piece_type(&self) -> PyPieceType {
        PyPieceType::from(self.inner.piece_type)
    }

    #[getter]
    fn piece_color(&self) -> PySide {
        PySide::from(self.inner.piece_color)
    }

    #[getter]
    fn column(&self) -> u8 {
        self.inner.column
    }

    #[getter]
    fn direction(&self) -> &str {
        match self.inner.direction {
            crate::move_notation::Direction::Forward => "Forward",
            crate::move_notation::Direction::Backward => "Backward",
            crate::move_notation::Direction::Horizontal => "Horizontal",
        }
    }

    #[getter]
    fn distance(&self) -> u8 {
        self.inner.distance
    }

    #[getter]
    fn qualifier(&self) -> Option<String> {
        match &self.inner.qualifier {
            Some(q) => Some(match q {
                Qualifier::Front => "Front".to_string(),
                Qualifier::Middle => "Middle".to_string(),
                Qualifier::Back => "Back".to_string(),
                Qualifier::Number(n) => format!("Number({})", n),
            }),
            None => None,
        }
    }

    /// Convert to Chinese notation
    fn to_chinese(&self, locale: PyChineseLocale) -> String {
        self.inner.to_chinese(locale.into())
    }

    /// Convert to WXF notation
    fn to_wxf(&self) -> String {
        self.inner.to_wxf()
    }

    fn __str__(&self) -> String {
        self.inner.to_chinese(ChineseLocale::Simplified)
    }
}
