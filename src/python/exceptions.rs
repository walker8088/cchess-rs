//! Custom Python exceptions for cchess-rs

use pyo3::create_exception;

// CChessError: general library exception
create_exception!(cchess, PyCChessError, pyo3::exceptions::PyException);

// EngineError: engine communication/execution exception
create_exception!(cchess, PyEngineError, pyo3::exceptions::PyException);
