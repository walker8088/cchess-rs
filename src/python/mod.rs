//! PyO3 Python bindings for cchess-rs
//!
//! This module provides Python bindings for the Chinese Chess (Xiangqi) library.
//! It is organized into focused sub-modules for maintainability.

// Suppress PyO3 0.20 macro warnings on newer Rust compilers
#![allow(non_local_definitions)]

// Sub-modules
mod board;
mod engine_driver;
mod engine_manager;
mod enums;
mod exceptions;
mod fen_cache;
mod file_formats;
mod game;
mod r#move;
mod move_notation;
mod movegen;
mod pgn;
mod utils;

use pyo3::prelude::*;

// Re-export types for use within the module
use board::PyBoard;
use engine_driver::{
    initial_fen, parse_bestmove_line, parse_info_line, parse_info_lines, resolve_engine_path,
    PyEngineOption, PyEngineProcess, PySearchInfo, PySearchResult,
};
use engine_manager::PyEngineManager;
use enums::{PyChineseLocale, PyEngineStatus, PyMoveFormat, PyPieceType, PySide};
use exceptions::{PyCChessError, PyEngineError};
use fen_cache::PyFenCache;
use file_formats::{
    board_from_xqf_bytes, board_to_xqf_bytes, read_cbl_buffer, read_cbl_file, read_cbr_buffer,
    read_cbr_file, read_xqf_file, write_xqf_file,
};
use game::{PyGame, PyGameMetadata};
use move_notation::PyMoveNotation;
use movegen::{
    generate_attack_matrix, generate_legal_moves, is_king_in_check, is_position_attacked,
};
use pgn::{game_to_pgn, parse_pgn, read_pgn_file, save_pgn_file};
use r#move::PyMoveNode;
use utils::{
    action_mirror, analyse_position, empty_board, empty_fen, fen_flip, fen_mirror,
    fen_mirror_engine, fen_move_color, fen_swap, fench_to_species, full_init_board, full_init_fen,
    get_fench_color, iccs2pos, iccs_flip, iccs_list_mirror, iccs_mirror, iccs_swap, play_move,
    pos2iccs, side_any, side_black, side_red,
};

/// Python module for cchess-rs
#[pymodule]
fn cchess_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    // Enums
    m.add_class::<PySide>()?;
    m.add_class::<PyPieceType>()?;
    m.add_class::<PyChineseLocale>()?;
    m.add_class::<PyMoveFormat>()?;

    // Core classes
    m.add_class::<PyBoard>()?;
    m.add_class::<PyGame>()?;
    m.add_class::<PyMoveNode>()?;
    m.add_class::<PyGameMetadata>()?;
    m.add_class::<PyMoveNotation>()?;

    // Engine classes
    m.add_class::<PyEngineStatus>()?;
    m.add_class::<PyEngineOption>()?;
    m.add_class::<PySearchInfo>()?;
    m.add_class::<PySearchResult>()?;
    m.add_class::<PyEngineProcess>()?;
    m.add_class::<PyFenCache>()?;
    m.add_class::<PyEngineManager>()?;

    // Exceptions
    m.add("CChessError", _py.get_type::<PyCChessError>())?;
    m.add("EngineError", _py.get_type::<PyEngineError>())?;

    // PGN functions
    m.add_function(wrap_pyfunction!(parse_pgn, m)?)?;
    m.add_function(wrap_pyfunction!(game_to_pgn, m)?)?;
    m.add_function(wrap_pyfunction!(read_pgn_file, m)?)?;
    m.add_function(wrap_pyfunction!(save_pgn_file, m)?)?;

    // XQF functions
    m.add_function(wrap_pyfunction!(read_xqf_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_xqf_file, m)?)?;
    m.add_function(wrap_pyfunction!(board_to_xqf_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(board_from_xqf_bytes, m)?)?;

    // CBR/CBL functions
    m.add_function(wrap_pyfunction!(read_cbr_file, m)?)?;
    m.add_function(wrap_pyfunction!(read_cbr_buffer, m)?)?;
    m.add_function(wrap_pyfunction!(read_cbl_file, m)?)?;
    m.add_function(wrap_pyfunction!(read_cbl_buffer, m)?)?;

    // Move generation
    m.add_function(wrap_pyfunction!(generate_legal_moves, m)?)?;

    // Attack matrix
    m.add_function(wrap_pyfunction!(generate_attack_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(is_position_attacked, m)?)?;
    m.add_function(wrap_pyfunction!(is_king_in_check, m)?)?;

    // Engine driver
    m.add_function(wrap_pyfunction!(resolve_engine_path, m)?)?;
    m.add_function(wrap_pyfunction!(parse_info_line, m)?)?;
    m.add_function(wrap_pyfunction!(parse_info_lines, m)?)?;
    m.add_function(wrap_pyfunction!(parse_bestmove_line, m)?)?;
    m.add_function(wrap_pyfunction!(initial_fen, m)?)?;

    // Constants
    m.add_function(wrap_pyfunction!(side_red, m)?)?;
    m.add_function(wrap_pyfunction!(side_black, m)?)?;
    m.add_function(wrap_pyfunction!(side_any, m)?)?;
    m.add_function(wrap_pyfunction!(full_init_fen, m)?)?;
    m.add_function(wrap_pyfunction!(empty_fen, m)?)?;
    m.add_function(wrap_pyfunction!(full_init_board, m)?)?;
    m.add_function(wrap_pyfunction!(empty_board, m)?)?;

    // Utility functions
    m.add_function(wrap_pyfunction!(fen_mirror, m)?)?;
    m.add_function(wrap_pyfunction!(fen_flip, m)?)?;
    m.add_function(wrap_pyfunction!(fen_swap, m)?)?;
    m.add_function(wrap_pyfunction!(fen_move_color, m)?)?;
    m.add_function(wrap_pyfunction!(pos2iccs, m)?)?;
    m.add_function(wrap_pyfunction!(iccs2pos, m)?)?;
    m.add_function(wrap_pyfunction!(iccs_mirror, m)?)?;
    m.add_function(wrap_pyfunction!(iccs_flip, m)?)?;
    m.add_function(wrap_pyfunction!(iccs_swap, m)?)?;
    m.add_function(wrap_pyfunction!(iccs_list_mirror, m)?)?;
    m.add_function(wrap_pyfunction!(get_fench_color, m)?)?;
    m.add_function(wrap_pyfunction!(fench_to_species, m)?)?;

    // Engine utility functions
    m.add_function(wrap_pyfunction!(action_mirror, m)?)?;
    m.add_function(wrap_pyfunction!(play_move, m)?)?;
    m.add_function(wrap_pyfunction!(analyse_position, m)?)?;
    m.add_function(wrap_pyfunction!(fen_mirror_engine, m)?)?;

    Ok(())
}
