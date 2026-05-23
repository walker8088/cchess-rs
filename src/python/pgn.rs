//! PGN parsing and serialization functions for Python bindings

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::game::Game;
use crate::pgn;

use super::game::PyGame;

/// Parse PGN string into a Game
#[pyfunction]
pub fn parse_pgn(pgn_text: &str) -> PyResult<PyGame> {
    let pgn_game = pgn::PGNParser::parse(pgn_text).map_err(|e| PyValueError::new_err(e))?;
    let mut game = Game::new();

    // Set metadata from tags
    if let Some(title) = pgn_game.tags.get("Title") {
        game.metadata.title = Some(title.clone());
    }
    if let Some(red) = pgn_game.tags.get("Red") {
        game.metadata.red_player = Some(red.clone());
    }
    if let Some(black) = pgn_game.tags.get("Black") {
        game.metadata.black_player = Some(black.clone());
    }
    if let Some(event) = pgn_game.tags.get("Event") {
        game.metadata.event = Some(event.clone());
    }
    if let Some(date) = pgn_game.tags.get("Date") {
        game.metadata.date = Some(date.clone());
    }
    if !pgn_game.result.is_empty() {
        game.metadata.result = Some(pgn_game.result.clone());
    }

    // Apply moves
    for pgn_move in &pgn_game.root_moves {
        if let (Some(from), Some(to)) = (pgn_move.from, pgn_move.to) {
            game.make_move(from, to)
                .map_err(|e| PyValueError::new_err(e))?;
        }
    }

    Ok(PyGame { inner: game })
}

/// Convert Game to PGN string
#[pyfunction]
pub fn game_to_pgn(game: &PyGame) -> String {
    game.inner.to_pgn()
}

/// Read PGN from file
#[pyfunction]
pub fn read_pgn_file(path: &str) -> PyResult<PyGame> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to read file: {}", e)))?;
    parse_pgn(&content)
}

/// Save Game to PGN file
#[pyfunction]
pub fn save_pgn_file(game: &PyGame, path: &str) -> PyResult<()> {
    let pgn = game.inner.to_pgn();
    std::fs::write(path, pgn)
        .map_err(|e| PyValueError::new_err(format!("Failed to write file: {}", e)))
}
