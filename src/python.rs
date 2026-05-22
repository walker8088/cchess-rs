/// PyO3 Python bindings for cchess-rs
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::board::Board;
use crate::game::{Game, GameMetadata, MoveNode};
use crate::move_notation::{ChineseLocale, MoveFormat, MoveNotation, Qualifier};
use crate::pgn;
use crate::pieces::{PieceType, Side};
use crate::xqf;

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
// Board Wrapper
// ============================================================================

#[pyclass(name = "Board")]
#[derive(Clone)]
pub struct PyBoard {
    inner: Board,
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

    /// Convert the board to a FEN string
    fn to_fen(&self) -> String {
        self.inner.to_fen()
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

// ============================================================================
// MoveNode Wrapper
// ============================================================================

#[pyclass(name = "MoveNode")]
#[derive(Clone)]
pub struct PyMoveNode {
    inner: MoveNode,
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
    inner: Game,
}

#[pymethods]
impl PyGame {
    #[new]
    fn new() -> Self {
        PyGame { inner: Game::new() }
    }

    /// Create a game from an existing board
    #[staticmethod]
    fn from_board(board: &PyBoard) -> Self {
        PyGame {
            inner: Game::from_board(board.inner.clone()),
        }
    }

    /// Get current board
    fn get_board(&self) -> PyBoard {
        PyBoard {
            inner: self.inner.board.clone(),
        }
    }

    /// Get current turn
    #[getter]
    fn current_turn(&self) -> PySide {
        PySide::from(self.inner.current_turn)
    }

    /// Check if game is over
    #[getter]
    fn is_game_over(&self) -> bool {
        self.inner.is_game_over
    }

    /// Get winner
    #[getter]
    fn winner(&self) -> Option<PySide> {
        self.inner.winner.map(PySide::from)
    }

    /// Get metadata
    #[getter]
    fn metadata(&self) -> PyGameMetadata {
        PyGameMetadata {
            inner: self.inner.metadata.clone(),
        }
    }

    #[setter]
    fn set_metadata(&mut self, metadata: PyGameMetadata) {
        self.inner.metadata = metadata.inner.clone();
    }

    /// Make a move
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

    /// Create a variation from the current position
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

    /// Navigate to a specific move
    fn navigate_to_move(&mut self, ply: u32) -> PyResult<()> {
        self.inner
            .navigate_to_move(ply)
            .map_err(|e| PyValueError::new_err(e))
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

    /// Get move tree as string
    fn get_move_tree_string(&self) -> String {
        self.inner.get_move_tree_string()
    }

    /// Get total moves in main line
    fn total_moves(&self) -> usize {
        self.inner.total_moves()
    }

    /// Get total variations
    fn total_variations(&self) -> u32 {
        self.inner.total_variations()
    }

    /// Convert game to PGN format
    fn to_pgn(&self) -> String {
        self.inner.to_pgn()
    }

    /// Check if current side is in check
    fn is_in_check(&self, side: PySide) -> bool {
        self.inner.is_in_check(side.into())
    }

    fn __str__(&self) -> String {
        self.inner.to_pgn()
    }

    fn __repr__(&self) -> String {
        format!(
            "Game(turn={:?}, moves={})",
            self.inner.current_turn,
            self.inner.total_moves()
        )
    }
}

// ============================================================================
// MoveNotation Wrapper
// ============================================================================

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

// ============================================================================
// PGN Functions
// ============================================================================

/// Parse PGN string into a Game
#[pyfunction]
fn parse_pgn(pgn_text: &str) -> PyResult<PyGame> {
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
fn game_to_pgn(game: &PyGame) -> String {
    game.inner.to_pgn()
}

/// Read PGN from file
#[pyfunction]
fn read_pgn_file(path: &str) -> PyResult<PyGame> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to read file: {}", e)))?;
    parse_pgn(&content)
}

/// Save Game to PGN file
#[pyfunction]
fn save_pgn_file(game: &PyGame, path: &str) -> PyResult<()> {
    let pgn = game.inner.to_pgn();
    std::fs::write(path, pgn)
        .map_err(|e| PyValueError::new_err(format!("Failed to write file: {}", e)))
}

// ============================================================================
// XQF Functions
// ============================================================================

/// Read XQF file and return a Game
#[pyfunction]
fn read_xqf_file(path: &str) -> PyResult<PyGame> {
    let xqf_file = xqf::read_xqf_with_variations(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to read XQF file: {:?}", e)))?;
    xqf::xqf_file_to_game(&xqf_file)
        .map(|game| PyGame { inner: game })
        .map_err(|e| PyValueError::new_err(format!("Failed to convert XQF to game: {:?}", e)))
}

/// Write Game to XQF file
#[pyfunction]
fn write_xqf_file(game: &PyGame, path: &str) -> PyResult<()> {
    xqf::write_xqf_from_game(&game.inner, path)
        .map_err(|e| PyValueError::new_err(format!("Failed to write XQF file: {:?}", e)))
}

/// Convert Board to XQF byte array
#[pyfunction]
fn board_to_xqf_bytes(board: &PyBoard) -> Vec<u8> {
    let data = xqf::board_to_xqf(&board.inner).unwrap_or([0u8; 90]);
    data.to_vec()
}

/// Create Board from XQF byte array
#[pyfunction]
fn board_from_xqf_bytes(data: Vec<u8>) -> PyResult<PyBoard> {
    if data.len() != 90 {
        return Err(PyValueError::new_err(
            "XQF board data must be exactly 90 bytes",
        ));
    }
    let mut arr = [0u8; 90];
    arr.copy_from_slice(&data);
    xqf::board_from_xqf(&arr)
        .map(|board| PyBoard { inner: board })
        .map_err(|e| PyValueError::new_err(format!("Invalid XQF data: {:?}", e)))
}

// ============================================================================
// Move Generation
// ============================================================================

/// Generate all legal moves for the current position
#[pyfunction]
fn generate_legal_moves(board: &PyBoard) -> Vec<(usize, usize, usize, usize)> {
    let side = if board.inner.to_fen().split_whitespace().nth(1) == Some("w") {
        Side::Red
    } else {
        Side::Black
    };

    let mut moves = Vec::new();
    for row in 0..10 {
        for col in 0..9 {
            if let Some((piece_type, piece_side)) = board.inner.get_piece_at(col, row) {
                if piece_side == side {
                    let piece_moves = crate::move_gen::generate_piece_moves(
                        &board.inner,
                        piece_type,
                        piece_side,
                        col,
                        row,
                    );
                    for m in piece_moves {
                        moves.push((m.from_col, m.from_row, m.to_col, m.to_row));
                    }
                }
            }
        }
    }
    moves
}

// ============================================================================
// Attack Matrix
// ============================================================================

/// Generate attack matrix for a side
#[pyfunction]
fn generate_attack_matrix(
    board: &PyBoard,
    side: PySide,
) -> Vec<Vec<Vec<(usize, usize, PyPieceType, PySide)>>> {
    let attacks = crate::attack_matrix::generate_attack_matrix(&board.inner, side.into());
    attacks
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|attackers| {
                    attackers
                        .into_iter()
                        .map(|(col, row, pt, s)| (col, row, PyPieceType::from(pt), PySide::from(s)))
                        .collect()
                })
                .collect()
        })
        .collect()
}

/// Check if a position is attacked by a side
#[pyfunction]
fn is_position_attacked(board: &PyBoard, col: usize, row: usize, side: PySide) -> bool {
    crate::attack_matrix::is_position_attacked(&board.inner, (col, row), side.into())
}

/// Check if a king is in check
#[pyfunction]
fn is_king_in_check(board: &PyBoard, side: PySide) -> bool {
    crate::attack_matrix::is_king_in_check(&board.inner, side.into())
}

// ============================================================================
// Engine Driver (UCI/UCCI Engine Process Management)
// ============================================================================

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

/// Engine option description (from `option` lines during init).
#[pyclass(name = "EngineOption")]
#[derive(Clone)]
pub struct PyEngineOption {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get)]
    pub default: Option<String>,
    #[pyo3(get)]
    pub min: Option<i64>,
    #[pyo3(get)]
    pub max: Option<i64>,
    #[pyo3(get)]
    pub var_values: Vec<String>,
}

/// Parsed search information from engine `info` lines.
#[pyclass(name = "SearchInfo")]
#[derive(Clone)]
pub struct PySearchInfo {
    #[pyo3(get, set)]
    pub depth: u32,
    #[pyo3(get, set)]
    pub seldepth: Option<u32>,
    #[pyo3(get, set)]
    pub time_ms: Option<u64>,
    #[pyo3(get, set)]
    pub nodes: Option<u64>,
    #[pyo3(get, set)]
    pub nps: Option<u64>,
    #[pyo3(get, set)]
    pub hashfull: Option<u32>,
    #[pyo3(get, set)]
    pub multipv: Option<u32>,
    #[pyo3(get)]
    pub score_cp: Option<i64>,
    #[pyo3(get)]
    pub score_mate: Option<i32>,
    #[pyo3(get, set)]
    pub currmove: Option<String>,
    #[pyo3(get, set)]
    pub currmovenumber: Option<u32>,
    #[pyo3(get)]
    pub pv: Vec<String>,
    #[pyo3(get, set)]
    pub root_moves: Option<u32>,
}

#[pymethods]
impl PySearchInfo {
    #[getter]
    fn is_mate(&self) -> bool {
        self.score_mate.is_some()
    }

    #[getter]
    fn score_value(&self) -> Option<i64> {
        self.score_cp
            .or_else(|| self.score_mate.map(|m| m as i64 * 100_000))
    }

    fn pv_string(&self) -> String {
        self.pv.join(" ")
    }

    fn __repr__(&self) -> String {
        format!(
            "SearchInfo(depth={}, score_cp={:?}, score_mate={:?}, nodes={:?}, pv={})",
            self.depth,
            self.score_cp,
            self.score_mate,
            self.nodes,
            self.pv_string()
        )
    }
}

impl From<crate::engine_driver::SearchInfo> for PySearchInfo {
    fn from(info: crate::engine_driver::SearchInfo) -> Self {
        let (score_cp, score_mate) = match info.score {
            Some(crate::engine_driver::Score::Cp(v)) => (Some(v), None),
            Some(crate::engine_driver::Score::Mate(v)) => (None, Some(v)),
            None => (None, None),
        };
        PySearchInfo {
            depth: info.depth,
            seldepth: info.seldepth,
            time_ms: info.time_ms,
            nodes: info.nodes,
            nps: info.nps,
            hashfull: info.hashfull,
            multipv: info.multipv,
            score_cp,
            score_mate,
            currmove: info.currmove,
            currmovenumber: info.currmovenumber,
            pv: info.pv,
            root_moves: info.root_moves,
        }
    }
}

/// Aggregated search result from an engine search.
#[pyclass(name = "SearchResult")]
#[derive(Clone)]
pub struct PySearchResult {
    #[pyo3(get)]
    pub bestmove: Option<String>,
    #[pyo3(get)]
    pub ponder: Option<String>,
    #[pyo3(get)]
    pub info_lines: Vec<PySearchInfo>,
    #[pyo3(get)]
    pub raw_lines: Vec<String>,
}

#[pymethods]
impl PySearchResult {
    /// Get the deepest search info line.
    #[getter]
    fn final_info(&self) -> Option<PySearchInfo> {
        self.info_lines.last().cloned()
    }

    /// Get the score from the deepest search.
    #[getter]
    fn score_cp(&self) -> Option<i64> {
        self.info_lines.last().and_then(|i| i.score_cp)
    }

    /// Get the mate distance if available.
    #[getter]
    fn score_mate(&self) -> Option<i32> {
        self.info_lines.last().and_then(|i| i.score_mate)
    }

    /// Check if the score is a mate.
    #[getter]
    fn is_mate(&self) -> bool {
        self.info_lines.last().map(|i| i.is_mate()).unwrap_or(false)
    }

    /// Get the nodes searched.
    #[getter]
    fn nodes(&self) -> Option<u64> {
        self.info_lines.last().and_then(|i| i.nodes)
    }

    /// Get the search time in ms.
    #[getter]
    fn time_ms(&self) -> Option<u64> {
        self.info_lines.last().and_then(|i| i.time_ms)
    }

    /// Get the nodes per second.
    #[getter]
    fn nps(&self) -> Option<u64> {
        self.info_lines.last().and_then(|i| i.nps)
    }

    /// Get the max depth reached.
    #[getter]
    fn depth(&self) -> Option<u32> {
        self.info_lines.last().map(|i| i.depth)
    }

    /// Get the principal variation as a string.
    fn pv_string(&self) -> String {
        self.info_lines
            .last()
            .map(|i| i.pv.join(" "))
            .unwrap_or_default()
    }

    fn __repr__(&self) -> String {
        format!(
            "SearchResult(bestmove={:?}, ponder={:?}, depth={:?}, nodes={:?}, pv={})",
            self.bestmove,
            self.ponder,
            self.depth(),
            self.nodes(),
            self.pv_string()
        )
    }
}

/// Synchronous engine process manager for UCI/UCCI engines.
///
/// This class spawns an external engine process and communicates with it
/// via stdin/stdout. It handles the UCI/UCCI protocol handshake and search.
///
/// Example:
///     engine = EngineProcess("path/to/engine.exe", "uci")
///     engine.init()
///     result = engine.search_movetime("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1", 2000)
///     print(result.bestmove)
///     engine.quit()
#[pyclass(name = "EngineProcess")]
pub struct PyEngineProcess {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    reader: Option<BufReader<ChildStdout>>,
    stderr_reader: Option<BufReader<std::process::ChildStderr>>,
    protocol: String,
    engine_name: String,
    engine_author: String,
    options: Vec<PyEngineOption>,
}

#[pymethods]
impl PyEngineProcess {
    /// Create a new engine process.
    ///
    /// Args:
    ///     exe_path: Path to the engine executable
    ///     protocol: "uci" or "ucci"
    #[new]
    fn new(exe_path: &str, protocol: &str) -> PyResult<Self> {
        let path = PathBuf::from(exe_path);
        if !path.exists() {
            return Err(PyValueError::new_err(format!(
                "Engine not found: {}",
                exe_path
            )));
        }

        let protocol_lower = protocol.to_lowercase();
        if protocol_lower != "uci" && protocol_lower != "ucci" {
            return Err(PyValueError::new_err(format!(
                "Invalid protocol: {}. Must be 'uci' or 'ucci'",
                protocol
            )));
        }

        let dir = path
            .parent()
            .ok_or_else(|| PyValueError::new_err("Engine path has no parent directory"))?
            .to_path_buf();

        let mut child = Command::new(&path)
            .current_dir(&dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PyValueError::new_err(format!("Failed to spawn engine: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PyValueError::new_err("Failed to capture stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PyValueError::new_err("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| PyValueError::new_err("Failed to capture stderr"))?;

        let reader = Some(BufReader::new(stdout));
        let stderr_reader = Some(BufReader::new(stderr));

        Ok(PyEngineProcess {
            child: Some(child),
            stdin: Some(stdin),
            reader,
            stderr_reader,
            protocol: protocol_lower,
            engine_name: String::new(),
            engine_author: String::new(),
            options: Vec::new(),
        })
    }

    /// Get the protocol type ("uci" or "ucci").
    #[getter]
    fn protocol(&self) -> &str {
        &self.protocol
    }

    /// Get the engine name (discovered after init).
    #[getter]
    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    /// Get the engine author (discovered after init).
    #[getter]
    fn engine_author(&self) -> &str {
        &self.engine_author
    }

    /// Get discovered engine options.
    #[getter]
    fn options(&self) -> Vec<PyEngineOption> {
        self.options.clone()
    }

    /// Send a raw command to the engine.
    fn send(&mut self, cmd: &str) -> PyResult<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Engine not connected"))?;
        writeln!(stdin, "{}", cmd)
            .map_err(|e| PyValueError::new_err(format!("Failed to write: {}", e)))
    }

    /// Read lines from the engine until a line starting with any of the given prefixes appears,
    /// or until the timeout elapses.
    fn read_until_any(&mut self, prefixes: Vec<&str>, timeout_ms: u64) -> PyResult<Vec<String>> {
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Engine not connected"))?;

        let start = Instant::now();
        let mut lines = Vec::new();
        let mut line_buf = String::new();

        loop {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return Err(PyValueError::new_err(format!(
                    "Timeout after {}ms waiting for line starting with {:?}",
                    timeout_ms, prefixes
                )));
            }

            line_buf.clear();
            let n = reader
                .read_line(&mut line_buf)
                .map_err(|e| PyValueError::new_err(format!("Read failed: {}", e)))?;
            if n == 0 {
                if lines
                    .iter()
                    .any(|l: &String| prefixes.iter().any(|p| l.starts_with(p)))
                {
                    break;
                }
                return Err(PyValueError::new_err("Engine closed stdout"));
            }
            let trimmed = line_buf.trim_end().to_string();
            lines.push(trimmed.clone());
            if prefixes.iter().any(|p| trimmed.starts_with(p)) {
                break;
            }
        }
        Ok(lines)
    }

    /// Drain pending stderr lines without blocking.
    fn drain_stderr(&mut self) -> Vec<String> {
        let mut output = Vec::new();
        if let Some(reader) = &mut self.stderr_reader {
            let mut line_buf = String::new();
            loop {
                line_buf.clear();
                match reader.read_line(&mut line_buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let trimmed = line_buf.trim_end().to_string();
                        if !trimmed.is_empty() {
                            output.push(trimmed);
                        }
                    }
                }
            }
        }
        output
    }

    /// Initialize the engine protocol (uci/ucci + isready).
    ///
    /// Returns list of all lines received during initialization.
    fn init(&mut self, timeout_ms: u64) -> PyResult<Vec<String>> {
        let protocol_cmd = if self.protocol == "uci" {
            "uci"
        } else {
            "ucci"
        };
        self.send(protocol_cmd)?;

        let ready_prefix = if self.protocol == "uci" {
            "uciok"
        } else {
            "ucciok"
        };

        let lines = self.read_until_any(vec![ready_prefix], timeout_ms)?;

        // Parse engine info
        for line in &lines {
            if let Some(rest) = line.strip_prefix("id ") {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "name" => self.engine_name = parts[1].to_string(),
                        "author" => self.engine_author = parts[1].to_string(),
                        _ => {}
                    }
                }
            }
            if let Some(rest) = line.strip_prefix("option ") {
                let opt = parse_engine_option(rest);
                self.options.push(opt);
            }
        }

        // Send isready
        self.send("isready")?;
        let more = self.read_until_any(vec!["readyok"], timeout_ms)?;
        let mut all = lines;
        all.extend(more);
        Ok(all)
    }

    /// Set a UCI option.
    fn setoption(&mut self, name: &str, value: &str) -> PyResult<()> {
        let cmd = if self.protocol == "uci" {
            format!("setoption name {} value {}", name, value)
        } else {
            format!("setoption {} {}", name, value)
        };
        self.send(&cmd)
    }

    /// Set a position by FEN string.
    fn position_fen(&mut self, fen: &str) -> PyResult<()> {
        self.send(&format!("position fen {}", fen))
    }

    /// Set a position with moves from startpos.
    fn position_startpos_moves(&mut self, moves: &str) -> PyResult<()> {
        self.send(&format!("position startpos moves {}", moves))
    }

    /// Search with time limit and return parsed SearchResult.
    ///
    /// Args:
    ///     fen: FEN string for the position
    ///     movetime_ms: Time limit in milliseconds
    ///     timeout_ms: Maximum time to wait for engine response
    ///
    /// Returns:
    ///     SearchResult with bestmove, info lines, etc.
    fn search_movetime(
        &mut self,
        fen: &str,
        movetime_ms: u64,
        timeout_ms: u64,
    ) -> PyResult<PySearchResult> {
        self.position_fen(fen)?;

        let go_cmd = if self.protocol == "uci" {
            format!("go movetime {}", movetime_ms)
        } else {
            // UCCI uses centiseconds
            format!("go time {}", movetime_ms / 10)
        };
        self.send(&go_cmd)?;

        let lines = self.read_until_any(vec!["bestmove", "nobestmove"], timeout_ms)?;
        self.drain_stderr();
        parse_search_result(&lines)
    }

    /// Search with depth limit and return parsed SearchResult.
    ///
    /// Args:
    ///     fen: FEN string for the position
    ///     depth: Maximum search depth
    ///     timeout_ms: Maximum time to wait for engine response
    ///
    /// Returns:
    ///     SearchResult with bestmove, info lines, etc.
    fn search_depth(&mut self, fen: &str, depth: u32, timeout_ms: u64) -> PyResult<PySearchResult> {
        self.position_fen(fen)?;
        self.send(&format!("go depth {}", depth))?;

        let lines = self.read_until_any(vec!["bestmove", "nobestmove"], timeout_ms)?;
        self.drain_stderr();
        parse_search_result(&lines)
    }

    /// Send quit command and terminate the engine.
    fn quit(&mut self) {
        let _ = self.send("quit");
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
        self.stdin = None;
        self.reader = None;
        self.stderr_reader = None;
    }

    fn __del__(&mut self) {
        self.quit();
    }

    fn __repr__(&self) -> String {
        format!(
            "EngineProcess(protocol='{}', name='{}', author='{}')",
            self.protocol, self.engine_name, self.engine_author
        )
    }
}

// ============================================================================
// Engine Helper Functions
// ============================================================================

/// Parse an `option` line into an EngineOption.
fn parse_engine_option(rest: &str) -> PyEngineOption {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    let mut name = String::new();
    let mut r#type = String::new();
    let mut default = None;
    let mut min = None;
    let mut max = None;
    let mut var_values = Vec::new();

    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "name" => {
                if let Some(&n) = parts.get(i + 1) {
                    name = n.to_string();
                }
                i += 2;
            }
            "type" => {
                if let Some(&t) = parts.get(i + 1) {
                    r#type = t.to_string();
                }
                i += 2;
            }
            "default" => {
                if r#type == "string" || r#type == "filename" {
                    let mut end = parts.len();
                    for j in (i + 1)..parts.len() {
                        if matches!(parts[j], "min" | "max" | "var") {
                            end = j;
                            break;
                        }
                    }
                    default = Some(parts[i + 1..end].join(" "));
                    i = end;
                } else {
                    if let Some(&d) = parts.get(i + 1) {
                        default = Some(d.to_string());
                    }
                    i += 2;
                }
            }
            "min" => {
                if let Some(&m) = parts.get(i + 1) {
                    min = m.parse().ok();
                }
                i += 2;
            }
            "max" => {
                if let Some(&m) = parts.get(i + 1) {
                    max = m.parse().ok();
                }
                i += 2;
            }
            "var" => {
                let mut end = i + 1;
                while end < parts.len() && parts[end] != "var" {
                    end += 1;
                }
                var_values.extend(parts[i + 1..end].iter().map(|s| s.to_string()));
                i = end;
            }
            _ => {
                i += 1;
            }
        }
    }

    PyEngineOption {
        name,
        r#type,
        default,
        min,
        max,
        var_values,
    }
}

/// Parse a single `info` line into a PySearchInfo.
fn parse_info_line_to_py(line: &str) -> Option<PySearchInfo> {
    crate::engine_driver::parse_info_line(line).map(PySearchInfo::from)
}

/// Parse all info lines from engine output into structured PySearchInfo.
fn parse_info_lines_to_py(lines: &[String]) -> Vec<PySearchInfo> {
    lines
        .iter()
        .filter(|l| l.starts_with("info "))
        .filter_map(|l| parse_info_line_to_py(l))
        .collect()
}

/// Parse bestmove line from engine output.
fn parse_bestmove_line_from_py(lines: &[String]) -> (Option<String>, Option<String>) {
    crate::engine_driver::parse_bestmove_line(lines)
}

/// Parse all search output into a PySearchResult.
fn parse_search_result(lines: &[String]) -> PyResult<PySearchResult> {
    let info_lines = parse_info_lines_to_py(lines);
    let (bestmove, ponder) = parse_bestmove_line_from_py(lines);

    if bestmove.is_none() && !lines.iter().any(|l| l == "nobestmove") {
        return Err(PyValueError::new_err(format!(
            "No bestmove found in engine output: {:?}",
            lines
        )));
    }

    Ok(PySearchResult {
        bestmove,
        ponder,
        info_lines,
        raw_lines: lines.to_vec(),
    })
}

/// Resolve an engine path from an environment variable or fall back to a default.
#[pyfunction]
fn resolve_engine_path(env_var: &str, default: &str) -> String {
    std::env::var(env_var).ok().unwrap_or_else(|| {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        format!("{}{}{}", manifest_dir, std::path::MAIN_SEPARATOR, default)
    })
}

/// Parse a single info line into a SearchInfo.
#[pyfunction]
fn parse_info_line(line: &str) -> Option<PySearchInfo> {
    parse_info_line_to_py(line)
}

/// Parse all info lines from a list of engine output lines.
#[pyfunction]
fn parse_info_lines(lines: Vec<String>) -> Vec<PySearchInfo> {
    parse_info_lines_to_py(&lines)
}

/// Parse bestmove from engine output lines.
#[pyfunction]
fn parse_bestmove_line(lines: Vec<String>) -> (Option<String>, Option<String>) {
    parse_bestmove_line_from_py(&lines)
}

/// Standard Chinese Chess initial position FEN string.
#[pyfunction]
fn initial_fen() -> String {
    crate::engine_driver::INITIAL_FEN.to_string()
}

// ============================================================================
// Python Module
// ============================================================================

#[pymodule]
fn cchess(_py: Python, m: &PyModule) -> PyResult<()> {
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

    // Move generation
    m.add_function(wrap_pyfunction!(generate_legal_moves, m)?)?;

    // Attack matrix
    m.add_function(wrap_pyfunction!(generate_attack_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(is_position_attacked, m)?)?;
    m.add_function(wrap_pyfunction!(is_king_in_check, m)?)?;

    // Engine driver
    m.add_class::<PyEngineOption>()?;
    m.add_class::<PySearchInfo>()?;
    m.add_class::<PySearchResult>()?;
    m.add_class::<PyEngineProcess>()?;
    m.add_function(wrap_pyfunction!(resolve_engine_path, m)?)?;
    m.add_function(wrap_pyfunction!(parse_info_line, m)?)?;
    m.add_function(wrap_pyfunction!(parse_info_lines, m)?)?;
    m.add_function(wrap_pyfunction!(parse_bestmove_line, m)?)?;
    m.add_function(wrap_pyfunction!(initial_fen, m)?)?;

    Ok(())
}
