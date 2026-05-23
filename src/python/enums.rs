//! Enum wrappers for Python bindings

use pyo3::prelude::*;

use crate::move_notation::{ChineseLocale, MoveFormat};
use crate::pieces::{PieceType, Side};

// ============================================================================
// Side Enum
// ============================================================================

#[pyclass(name = "Side")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PySide {
    Any,
    Red,
    Black,
}

#[pymethods]
impl PySide {
    fn __hash__(&self) -> u64 {
        match self {
            PySide::Any => 0,
            PySide::Red => 1,
            PySide::Black => 2,
        }
    }
}

impl From<PySide> for Side {
    fn from(side: PySide) -> Self {
        match side {
            PySide::Any => Side::Any,
            PySide::Red => Side::Red,
            PySide::Black => Side::Black,
        }
    }
}

impl From<Side> for PySide {
    fn from(side: Side) -> Self {
        match side {
            Side::Any => PySide::Any,
            Side::Red => PySide::Red,
            Side::Black => PySide::Black,
        }
    }
}

// ============================================================================
// PieceType Enum
// ============================================================================

#[pyclass(name = "PieceType")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PyPieceType {
    King,
    Advisor,
    Elephant,
    Knight,
    Rook,
    Cannon,
    Pawn,
}

#[pymethods]
impl PyPieceType {
    fn __hash__(&self) -> u64 {
        match self {
            PyPieceType::King => 0,
            PyPieceType::Advisor => 1,
            PyPieceType::Elephant => 2,
            PyPieceType::Knight => 3,
            PyPieceType::Rook => 4,
            PyPieceType::Cannon => 5,
            PyPieceType::Pawn => 6,
        }
    }
}

impl From<PyPieceType> for PieceType {
    fn from(pt: PyPieceType) -> Self {
        match pt {
            PyPieceType::King => PieceType::King,
            PyPieceType::Advisor => PieceType::Advisor,
            PyPieceType::Elephant => PieceType::Elephant,
            PyPieceType::Knight => PieceType::Knight,
            PyPieceType::Rook => PieceType::Rook,
            PyPieceType::Cannon => PieceType::Cannon,
            PyPieceType::Pawn => PieceType::Pawn,
        }
    }
}

impl From<PieceType> for PyPieceType {
    fn from(pt: PieceType) -> Self {
        match pt {
            PieceType::King => PyPieceType::King,
            PieceType::Advisor => PyPieceType::Advisor,
            PieceType::Elephant => PyPieceType::Elephant,
            PieceType::Knight => PyPieceType::Knight,
            PieceType::Rook => PyPieceType::Rook,
            PieceType::Cannon => PyPieceType::Cannon,
            PieceType::Pawn => PyPieceType::Pawn,
        }
    }
}

// ============================================================================
// ChineseLocale Enum
// ============================================================================

#[pyclass(name = "ChineseLocale")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PyChineseLocale {
    Simplified,
    Traditional,
}

#[pymethods]
impl PyChineseLocale {
    fn __hash__(&self) -> u64 {
        match self {
            PyChineseLocale::Simplified => 0,
            PyChineseLocale::Traditional => 1,
        }
    }
}

impl From<PyChineseLocale> for ChineseLocale {
    fn from(locale: PyChineseLocale) -> Self {
        match locale {
            PyChineseLocale::Simplified => ChineseLocale::Simplified,
            PyChineseLocale::Traditional => ChineseLocale::Traditional,
        }
    }
}

// ============================================================================
// MoveFormat Enum
// ============================================================================

#[pyclass(name = "MoveFormat")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PyMoveFormat {
    Chinese,
    WXF,
    ICCS,
}

#[pymethods]
impl PyMoveFormat {
    fn __hash__(&self) -> u64 {
        match self {
            PyMoveFormat::Chinese => 0,
            PyMoveFormat::WXF => 1,
            PyMoveFormat::ICCS => 2,
        }
    }
}

impl From<PyMoveFormat> for MoveFormat {
    fn from(fmt: PyMoveFormat) -> Self {
        match fmt {
            PyMoveFormat::Chinese => MoveFormat::Chinese,
            PyMoveFormat::WXF => MoveFormat::WXF,
            PyMoveFormat::ICCS => MoveFormat::ICCS,
        }
    }
}

// ============================================================================
// EngineStatus Enum
// ============================================================================

/// Engine running status (matches Python cchess EngineStatus IntEnum)
#[pyclass(name = "EngineStatus")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PyEngineStatus {
    Error,
    Booting,
    Ready,
    Waiting,
    InfoMove,
    Move,
    Dead,
    Unknown,
    BoardReset,
}

#[pymethods]
impl PyEngineStatus {
    fn __hash__(&self) -> u64 {
        match self {
            PyEngineStatus::Error => 0,
            PyEngineStatus::Booting => 1,
            PyEngineStatus::Ready => 2,
            PyEngineStatus::Waiting => 3,
            PyEngineStatus::InfoMove => 4,
            PyEngineStatus::Move => 5,
            PyEngineStatus::Dead => 6,
            PyEngineStatus::Unknown => 7,
            PyEngineStatus::BoardReset => 8,
        }
    }

    /// Convert to integer value (IntEnum compatible)
    fn __int__(&self) -> i32 {
        match self {
            PyEngineStatus::Error => 0,
            PyEngineStatus::Booting => 1,
            PyEngineStatus::Ready => 2,
            PyEngineStatus::Waiting => 3,
            PyEngineStatus::InfoMove => 4,
            PyEngineStatus::Move => 5,
            PyEngineStatus::Dead => 6,
            PyEngineStatus::Unknown => 7,
            PyEngineStatus::BoardReset => 8,
        }
    }
}
