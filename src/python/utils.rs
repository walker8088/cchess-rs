//! Utility functions for FEN/ICCS manipulation and engine operations

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::exceptions::PyEngineError;

// ============================================================================
// Constants
// ============================================================================

/// Red side constant (1)
#[pyfunction]
pub fn side_red() -> i32 {
    1
}

/// Black side constant (2)
#[pyfunction]
pub fn side_black() -> i32 {
    2
}

/// Any side constant (0)
#[pyfunction]
pub fn side_any() -> i32 {
    0
}

/// Full initial position FEN
#[pyfunction]
pub fn full_init_fen() -> &'static str {
    "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w"
}

/// Empty board FEN
#[pyfunction]
pub fn empty_fen() -> &'static str {
    "9/9/9/9/9/9/9/9/9/9 w"
}

/// Full initial position board part (without side to move)
#[pyfunction]
pub fn full_init_board() -> &'static str {
    "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR"
}

/// Empty board part
#[pyfunction]
pub fn empty_board() -> &'static str {
    "9/9/9/9/9/9/9/9/9/9"
}

// ============================================================================
// FEN Manipulation
// ============================================================================

/// Mirror FEN horizontally (left-right flip)
#[pyfunction]
pub fn fen_mirror(fen: &str) -> String {
    let parts: Vec<&str> = fen.splitn(2, ' ').collect();
    let board_part = parts[0];
    let side_part = if parts.len() > 1 { parts[1] } else { "" };

    let mut mirrored_rows = Vec::new();
    for row_str in board_part.split('/') {
        let mut mirrored = String::new();
        let mut chars: Vec<char> = row_str.chars().collect();
        chars.reverse();
        for c in chars {
            mirrored.push(c);
        }
        mirrored_rows.push(mirrored);
    }
    let result = mirrored_rows.join("/");
    if side_part.is_empty() {
        result
    } else {
        format!("{} {}", result, side_part)
    }
}

/// Flip FEN vertically (swap red and black sides)
#[pyfunction]
pub fn fen_flip(fen: &str) -> String {
    let parts: Vec<&str> = fen.splitn(2, ' ').collect();
    let board_part = parts[0];
    let side_part = if parts.len() > 1 { parts[1] } else { "" };

    let rows: Vec<&str> = board_part.split('/').collect();
    let mut flipped_rows: Vec<String> = rows.iter().rev().map(|r| r.to_string()).collect();

    for row in &mut flipped_rows {
        let mut new_row = String::new();
        for c in row.chars() {
            if c.is_ascii_uppercase() {
                new_row.push(c.to_ascii_lowercase());
            } else if c.is_ascii_lowercase() {
                new_row.push(c.to_ascii_uppercase());
            } else {
                new_row.push(c);
            }
        }
        *row = new_row;
    }

    let new_side = match side_part.split_whitespace().next() {
        Some("w") => "b",
        Some("b") => "w",
        _ => "w",
    };

    format!("{} {}", flipped_rows.join("/"), new_side)
}

/// Swap FEN colors (red <-> black) without flipping board
#[pyfunction]
pub fn fen_swap(fen: &str) -> String {
    let parts: Vec<&str> = fen.splitn(2, ' ').collect();
    let board_part = parts[0];
    let side_part = if parts.len() > 1 { parts[1] } else { "" };

    let mut swapped = String::new();
    for c in board_part.chars() {
        if c.is_ascii_uppercase() {
            swapped.push(c.to_ascii_lowercase());
        } else if c.is_ascii_lowercase() {
            swapped.push(c.to_ascii_uppercase());
        } else {
            swapped.push(c);
        }
    }

    let new_side = match side_part.split_whitespace().next() {
        Some("w") => "b",
        Some("b") => "w",
        _ => "w",
    };

    format!("{} {}", swapped, new_side)
}

/// FEN move color: get the side to move from FEN
#[pyfunction]
pub fn fen_move_color(fen: &str) -> i32 {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() < 2 {
        return 0;
    }
    match parts[1] {
        "w" => 1,
        "b" => 2,
        _ => 0,
    }
}

// ============================================================================
// ICCS Conversion
// ============================================================================

/// Convert position to ICCS notation (e.g., (4,2) -> "e2")
#[pyfunction]
pub fn pos2iccs(from_col: usize, from_row: usize, to_col: usize, to_row: usize) -> String {
    format!(
        "{}{}{}{}",
        (b'a' + from_col as u8) as char,
        from_row,
        (b'a' + to_col as u8) as char,
        to_row
    )
}

/// Parse ICCS notation to positions
#[pyfunction]
pub fn iccs2pos(iccs: &str) -> PyResult<((usize, usize), (usize, usize))> {
    if iccs.len() != 4 {
        return Err(PyValueError::new_err(format!(
            "Invalid ICCS notation: {}",
            iccs
        )));
    }
    let bytes = iccs.as_bytes();
    let from_col = (bytes[0].to_ascii_lowercase() - b'a') as usize;
    let from_row = (bytes[1] - b'0') as usize;
    let to_col = (bytes[2].to_ascii_lowercase() - b'a') as usize;
    let to_row = (bytes[3] - b'0') as usize;
    Ok(((from_col, from_row), (to_col, to_row)))
}

/// Mirror ICCS notation horizontally
#[pyfunction]
pub fn iccs_mirror(iccs: &str) -> PyResult<String> {
    if iccs.len() != 4 {
        return Err(PyValueError::new_err(format!(
            "Invalid ICCS notation: {}",
            iccs
        )));
    }
    let bytes = iccs.as_bytes();
    let mirror_char = |c: u8| -> char { ('i' as u8 - c + 'a' as u8) as char };
    Ok(format!(
        "{}{}{}{}",
        mirror_char(bytes[0]),
        bytes[1] as char,
        mirror_char(bytes[2]),
        bytes[3] as char
    ))
}

/// Flip ICCS notation vertically
#[pyfunction]
pub fn iccs_flip(iccs: &str) -> PyResult<String> {
    if iccs.len() != 4 {
        return Err(PyValueError::new_err(format!(
            "Invalid ICCS notation: {}",
            iccs
        )));
    }
    let bytes = iccs.as_bytes();
    let flip_row = |r: u8| -> char { (9 - (r - b'0')) as u8 as char };
    Ok(format!(
        "{}{}{}{}",
        bytes[0] as char,
        flip_row(bytes[1]),
        bytes[2] as char,
        flip_row(bytes[3])
    ))
}

/// Swap ICCS notation (mirror + flip)
#[pyfunction]
pub fn iccs_swap(iccs: &str) -> PyResult<String> {
    if iccs.len() != 4 {
        return Err(PyValueError::new_err(format!(
            "Invalid ICCS notation: {}",
            iccs
        )));
    }
    let bytes = iccs.as_bytes();
    let mirror_char = |c: u8| -> char { ('i' as u8 - c + 'a' as u8) as char };
    let flip_row = |r: u8| -> char { (9 - (r - b'0')) as u8 as char };
    Ok(format!(
        "{}{}{}{}",
        mirror_char(bytes[0]),
        flip_row(bytes[1]),
        mirror_char(bytes[2]),
        flip_row(bytes[3])
    ))
}

/// Mirror a list of ICCS moves (horizontally flip each move)
#[pyfunction]
pub fn iccs_list_mirror(iccs_list: Vec<String>) -> PyResult<Vec<String>> {
    iccs_list.iter().map(|iccs| iccs_mirror(iccs)).collect()
}

// ============================================================================
// FEN Character Helpers
// ============================================================================

/// Get FEN character color: 1 for red (uppercase), 2 for black (lowercase)
#[pyfunction]
pub fn get_fench_color(fench: &str) -> Option<i32> {
    let c = fench.chars().next()?;
    if c.is_ascii_uppercase() {
        Some(1)
    } else if c.is_ascii_lowercase() {
        Some(2)
    } else {
        None
    }
}

/// Get FEN character type (lowercase species)
#[pyfunction]
pub fn fench_to_species(fench: &str) -> Option<(String, i32)> {
    let c = fench.chars().next()?;
    if c == '.' {
        return None;
    }
    let species = c.to_lowercase().to_string();
    let color = if c.is_ascii_uppercase() { 1 } else { 2 };
    Some((species, color))
}

// ============================================================================
// Engine Utility Functions
// ============================================================================

/// Mirror action fields (move, ponder, moves) in a dict.
/// Used when retrieving actions from mirrored FEN cache.
#[pyfunction]
pub fn action_mirror(_py: Python<'_>, action: &PyDict) -> PyResult<Py<PyDict>> {
    let result = action.copy()?;

    for key in ["move", "ponder"] {
        if let Ok(Some(val)) = result.get_item(key) {
            let s = val.to_string();
            if let Ok(mirrored) = iccs_mirror(&s) {
                let _ = result.set_item(key, mirrored);
            }
        }
    }

    if let Ok(Some(val)) = result.get_item("moves") {
        if let Ok(iter) = val.iter() {
            let moves: Vec<String> = iter
                .filter_map(|item| item.ok().and_then(|x| x.extract::<String>().ok()))
                .collect();
            if !moves.is_empty() {
                if let Ok(mirrored) = iccs_list_mirror(moves) {
                    let _ = result.set_item("moves", mirrored);
                }
            }
        }
    }

    Ok(result.into())
}

/// Play a move against the engine (synchronous convenience function).
#[pyfunction]
#[pyo3(signature = (engine_path, fen, *, protocol="uci", depth=10, movetime_ms=None, options=None))]
pub fn play_move(
    engine_path: &str,
    fen: &str,
    protocol: &str,
    depth: u32,
    movetime_ms: Option<u32>,
    options: Option<Vec<(String, String)>>,
) -> PyResult<String> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    let engine_path_resolved =
        super::engine_driver::resolve_engine_path("CCHESS_ENGINE", engine_path);

    let mut child = Command::new(&engine_path_resolved)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| PyEngineError::new_err(format!("Failed to start engine: {}", e)))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| PyEngineError::new_err("Failed to get engine stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| PyEngineError::new_err("Failed to get engine stdout"))?;
    let reader = BufReader::new(stdout);

    let init_cmd = match protocol {
        "uci" => "uci\n",
        _ => "ucci\n",
    };
    let ok_resp = match protocol {
        "uci" => "uciok",
        _ => "ucciok",
    };

    stdin
        .write_all(init_cmd.as_bytes())
        .map_err(|e| PyEngineError::new_err(format!("Failed to send init command: {}", e)))?;
    stdin
        .flush()
        .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;

    if let Some(opts) = options {
        for (name, value) in opts {
            let cmd = format!("setoption name {} value {}\n", name, value);
            stdin
                .write_all(cmd.as_bytes())
                .map_err(|e| PyEngineError::new_err(format!("Failed to set option: {}", e)))?;
            stdin
                .flush()
                .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;
        }
    }

    let pos_cmd = format!("position fen {}\n", fen);
    stdin
        .write_all(pos_cmd.as_bytes())
        .map_err(|e| PyEngineError::new_err(format!("Failed to send position: {}", e)))?;
    stdin
        .flush()
        .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;

    let go_cmd = if let Some(ms) = movetime_ms {
        format!("go depth {} movetime {}\n", depth, ms)
    } else {
        format!("go depth {}\n", depth)
    };
    stdin
        .write_all(go_cmd.as_bytes())
        .map_err(|e| PyEngineError::new_err(format!("Failed to send go: {}", e)))?;
    stdin
        .flush()
        .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;

    drop(stdin);

    let mut bestmove = None;
    let mut saw_ok = false;
    for line in reader.lines() {
        let line = line.map_err(|e| PyEngineError::new_err(format!("Read error: {}", e)))?;
        if !saw_ok {
            if line == ok_resp {
                saw_ok = true;
            }
            continue;
        }
        if line.starts_with("bestmove") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                bestmove = Some(parts[1].to_string());
            }
            break;
        }
    }

    let _ = child.wait();

    bestmove.ok_or_else(|| PyEngineError::new_err("Engine did not return a bestmove"))
}

/// Analyse a position using the engine (synchronous convenience function).
#[pyfunction]
#[pyo3(signature = (engine_path, fen, *, protocol="uci", depth=20, movetime_ms=None, multipv=1, options=None))]
pub fn analyse_position(
    engine_path: &str,
    fen: &str,
    protocol: &str,
    depth: u32,
    movetime_ms: Option<u32>,
    multipv: u32,
    options: Option<Vec<(String, String)>>,
) -> PyResult<Vec<PyObject>> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    let engine_path_resolved =
        super::engine_driver::resolve_engine_path("CCHESS_ENGINE", engine_path);

    let mut child = Command::new(&engine_path_resolved)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| PyEngineError::new_err(format!("Failed to start engine: {}", e)))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| PyEngineError::new_err("Failed to get engine stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| PyEngineError::new_err("Failed to get engine stdout"))?;
    let reader = BufReader::new(stdout);

    let init_cmd = match protocol {
        "uci" => "uci\n",
        _ => "ucci\n",
    };
    let ok_resp = match protocol {
        "uci" => "uciok",
        _ => "ucciok",
    };

    stdin
        .write_all(init_cmd.as_bytes())
        .map_err(|e| PyEngineError::new_err(format!("Failed to send init command: {}", e)))?;
    stdin
        .flush()
        .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;

    if let Some(opts) = options {
        for (name, value) in opts {
            let cmd = format!("setoption name {} value {}\n", name, value);
            stdin
                .write_all(cmd.as_bytes())
                .map_err(|e| PyEngineError::new_err(format!("Failed to set option: {}", e)))?;
            stdin
                .flush()
                .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;
        }
    }

    if multipv > 1 {
        let cmd = format!("setoption name MultiPV value {}\n", multipv);
        stdin
            .write_all(cmd.as_bytes())
            .map_err(|e| PyEngineError::new_err(format!("Failed to set MultiPV: {}", e)))?;
        stdin
            .flush()
            .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;
    }

    let pos_cmd = format!("position fen {}\n", fen);
    stdin
        .write_all(pos_cmd.as_bytes())
        .map_err(|e| PyEngineError::new_err(format!("Failed to send position: {}", e)))?;
    stdin
        .flush()
        .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;

    let go_cmd = if let Some(ms) = movetime_ms {
        format!("go depth {} movetime {}\n", depth, ms)
    } else {
        format!("go depth {}\n", depth)
    };
    stdin
        .write_all(go_cmd.as_bytes())
        .map_err(|e| PyEngineError::new_err(format!("Failed to send go: {}", e)))?;
    stdin
        .flush()
        .map_err(|e| PyEngineError::new_err(format!("Failed to flush: {}", e)))?;

    drop(stdin);

    Ok(Python::with_gil(|py| {
        let mut results: Vec<PyObject> = Vec::new();
        let mut saw_ok = false;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            if !saw_ok {
                if line == ok_resp {
                    saw_ok = true;
                }
                continue;
            }
            if line.starts_with("bestmove") {
                break;
            }
            if line.starts_with("info") {
                if let Some(info) = super::engine_driver::parse_info_line_to_py(&line) {
                    let info_obj = Py::new(py, info).ok().map(|p| p.into_py(py));
                    if let Some(obj) = info_obj {
                        results.push(obj);
                    }
                }
            }
        }
        results
    }))
}

/// Mirror a FEN string horizontally (left-right flip) - Engine utility version
#[pyfunction]
pub fn fen_mirror_engine(fen: &str) -> String {
    fen_mirror(fen)
}
