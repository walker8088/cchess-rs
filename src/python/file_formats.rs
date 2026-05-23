//! CBR/CBL/XQF file format functions for Python bindings

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::xqf;

use super::game::PyGame;

/// Read CBR file and return a Game
#[pyfunction]
pub fn read_cbr_file(path: &str) -> PyResult<PyGame> {
    crate::cbr::read_from_cbr(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to read CBR file: {}", e)))?
        .map(|game| PyGame { inner: game })
        .ok_or_else(|| PyValueError::new_err("Invalid CBR file or not a CBR format"))
}

/// Read CBR file from bytes
#[pyfunction]
pub fn read_cbr_buffer(data: Vec<u8>) -> PyResult<PyGame> {
    crate::cbr::read_from_cbr_buffer(&data)
        .map(|game| PyGame { inner: game })
        .ok_or_else(|| PyValueError::new_err("Invalid CBR data or not a CBR format"))
}

/// Read CBL library file and return (name, games_list)
#[pyfunction]
pub fn read_cbl_file(path: &str) -> PyResult<(String, Vec<PyGame>)> {
    let library = crate::cbr::read_from_cbl(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to read CBL file: {}", e)))?
        .ok_or_else(|| PyValueError::new_err("Invalid CBL file or not a CBL format"))?;
    let games = library
        .games
        .into_iter()
        .map(|g| PyGame { inner: g })
        .collect();
    Ok((library.name, games))
}

/// Read CBL library from bytes
#[pyfunction]
pub fn read_cbl_buffer(data: Vec<u8>) -> PyResult<(String, Vec<PyGame>)> {
    let library = crate::cbr::read_from_cbl_buffer(&data)
        .ok_or_else(|| PyValueError::new_err("Invalid CBL data or not a CBL format"))?;
    let games = library
        .games
        .into_iter()
        .map(|g| PyGame { inner: g })
        .collect();
    Ok((library.name, games))
}

/// Read XQF file and return a Game
#[pyfunction]
pub fn read_xqf_file(path: &str) -> PyResult<PyGame> {
    let xqf_file = xqf::read_xqf_with_variations(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to read XQF file: {:?}", e)))?;
    xqf::xqf_file_to_game(&xqf_file)
        .map(|game| PyGame { inner: game })
        .map_err(|e| PyValueError::new_err(format!("Failed to convert XQF to game: {:?}", e)))
}

/// Write Game to XQF file
#[pyfunction]
pub fn write_xqf_file(game: &PyGame, path: &str) -> PyResult<()> {
    xqf::write_xqf_from_game(&game.inner, path)
        .map_err(|e| PyValueError::new_err(format!("Failed to write XQF file: {:?}", e)))
}

/// Convert Board to XQF byte array
#[pyfunction]
pub fn board_to_xqf_bytes(board: &super::board::PyBoard) -> Vec<u8> {
    let data = xqf::board_to_xqf(&board.inner).unwrap_or([0u8; 90]);
    data.to_vec()
}

/// Create Board from XQF byte array
#[pyfunction]
pub fn board_from_xqf_bytes(data: Vec<u8>) -> PyResult<super::board::PyBoard> {
    if data.len() != 90 {
        return Err(PyValueError::new_err(
            "XQF board data must be exactly 90 bytes",
        ));
    }
    let mut arr = [0u8; 90];
    arr.copy_from_slice(&data);
    xqf::board_from_xqf(&arr)
        .map(|board| super::board::PyBoard { inner: board })
        .map_err(|e| PyValueError::new_err(format!("Invalid XQF data: {:?}", e)))
}
