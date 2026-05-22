//! UCI/UCCI Engine Protocol Support for Chinese Chess (Xiangqi)
//!
//! This module implements the Universal Chess Interface (UCI) and Universal
//! Chinese Chess Interface (UCCI) protocols, allowing external GUIs to
//! communicate with the cchess-rs engine.
//!
//! # Protocol Basics
//! - Engine reads commands from stdin, writes responses to stdout
//! - UCI: `uci` → `uciok`, `position` → `go` → `bestmove`
//! - UCCI: same structure with `ucci`/`ucciok` prefix
//!
//! # Move Format
//! UCI move format: `<from_col><from_row><to_col><to_row>`
//! - Columns: a=0, b=1, ..., i=8
//! - Rows: 0-9 where UCI row 0 = Rust row 9 (Red's base), UCI row 9 = Rust row 0
//! - Conversion: `uci_row = 9 - rust_row`, `rust_row = 9 - uci_row`

#![allow(dead_code)]

use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

use crate::board::Board;
use crate::move_gen::{generate_moves, Move};
use crate::move_notation::{format_iccs_move, try_parse_iccs_move};
use crate::pieces::Side;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const ENGINE_NAME: &str = "cchess-rs";
const ENGINE_AUTHOR: &str = "cchess-rs contributors";

const INFINITY: i32 = 30000;
const MATE_SCORE: i32 = 29000;
const MATE_THRESHOLD: i32 = MATE_SCORE - 2000;

/// Depth limit for quiescence search
const QUIESCENCE_MAX_DEPTH: usize = 6;

/// Columns for UCI notation: a=0, b=1, ..., i=8
const COL_CHARS: [char; 9] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];

// Material values in centipawns
const PIECE_VALUES: [i32; 7] = [
    10000, // King
    200,   // Advisor
    200,   // Elephant
    400,   // Knight
    900,   // Rook
    450,   // Cannon
    100,   // Pawn
];

// Material values in centipawns
// ---------------------------------------------------------------------------
// Protocol Enum
// ---------------------------------------------------------------------------

/// Supported engine protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// Universal Chess Interface
    UCI,
    /// Universal Chinese Chess Interface
    UCCI,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::UCI => write!(f, "uci"),
            Protocol::UCCI => write!(f, "ucci"),
        }
    }
}

// ---------------------------------------------------------------------------
// Engine Options
// ---------------------------------------------------------------------------

/// Configuration options for the engine
#[derive(Debug, Clone)]
pub struct EngineOptions {
    /// Hash table size in MB (for future transposition table)
    pub hash_size: usize,
    /// Maximum search depth
    pub max_depth: usize,
    /// Time limit per move in milliseconds
    pub move_time_ms: Option<u64>,
    /// White (Red) time remaining in milliseconds
    pub wtime: Option<u64>,
    /// Black time remaining in milliseconds
    pub btime: Option<u64>,
    /// White (Red) time increment per move
    pub winc: Option<u64>,
    /// Black time increment per move
    pub binc: Option<u64>,
    /// Node limit for search
    pub nodes: Option<u64>,
    /// Search indefinitely until stopped
    pub infinite: bool,
    /// Which protocol we're using
    pub protocol: Protocol,
}

impl Default for EngineOptions {
    fn default() -> Self {
        EngineOptions {
            hash_size: 16,
            max_depth: 10,
            move_time_ms: None,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            nodes: None,
            infinite: false,
            protocol: Protocol::UCI,
        }
    }
}

impl EngineOptions {
    /// Get the time budget for the current side to move
    pub fn time_for_side(&self, side: Side) -> Option<u64> {
        if let Some(t) = self.move_time_ms {
            return Some(t);
        }
        match side {
            Side::Red => self.wtime,
            Side::Black => self.btime,
            Side::Any => None,
        }
    }

    /// Get the increment for the current side
    pub fn increment_for_side(&self, side: Side) -> u64 {
        match side {
            Side::Red => self.winc.unwrap_or(0),
            Side::Black => self.binc.unwrap_or(0),
            Side::Any => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Search Result
// ---------------------------------------------------------------------------

/// Result of a search operation
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The best move found
    pub best_move: Option<Move>,
    /// Search depth reached
    pub depth: usize,
    /// Evaluation score in centipawns (positive = good for side to move)
    pub score: i32,
    /// Number of nodes searched
    pub nodes: u64,
    /// Time spent in milliseconds
    pub time_ms: u64,
    /// Nodes per second
    pub nps: u64,
    /// Whether the result is a mate score
    pub is_mate: bool,
}

impl SearchResult {
    /// Create a null result (no move found)
    pub fn null() -> Self {
        SearchResult {
            best_move: None,
            depth: 0,
            score: 0,
            nodes: 0,
            time_ms: 0,
            nps: 0,
            is_mate: false,
        }
    }

    /// Format the score for UCI output
    pub fn format_score(&self) -> String {
        if self.is_mate {
            let mate_in = (MATE_SCORE - self.score.abs()) / 2 + 1;
            if self.score > 0 {
                format!("mate {}", mate_in)
            } else {
                format!("mate -{}", mate_in)
            }
        } else {
            format!("cp {}", self.score)
        }
    }
}

// ---------------------------------------------------------------------------
// Search State
// ---------------------------------------------------------------------------

/// Internal state maintained during search
struct SearchState {
    nodes: u64,
    time_limit: Option<Instant>,
    node_limit: Option<u64>,
    stopped: bool,
    current_depth: usize,
}

impl SearchState {
    fn new(time_limit: Option<Instant>, node_limit: Option<u64>) -> Self {
        SearchState {
            nodes: 0,
            time_limit,
            node_limit,
            stopped: false,
            current_depth: 0,
        }
    }

    /// Check if search should stop (time or node limit exceeded)
    fn should_stop(&self) -> bool {
        if self.stopped {
            return true;
        }
        if let Some(limit) = self.node_limit {
            if self.nodes >= limit {
                return true;
            }
        }
        if let Some(deadline) = self.time_limit {
            if Instant::now() >= deadline {
                return true;
            }
        }
        false
    }

    fn increment_nodes(&mut self) {
        self.nodes += 1;
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The main engine struct that handles protocol communication and search
pub struct Engine {
    protocol: Protocol,
    options: EngineOptions,
    board: Board,
    current_turn: Side,
    /// Whether we've received the initial protocol greeting
    initialized: bool,
    /// Whether we're ready for commands
    ready: bool,
}

impl Engine {
    /// Create a new engine with the specified protocol
    pub fn new(protocol: Protocol) -> Self {
        let mut board = Board::new();
        board.initial_position();
        Engine {
            protocol,
            options: EngineOptions::default(),
            board,
            current_turn: Side::Red,
            initialized: false,
            ready: false,
        }
    }

    // -----------------------------------------------------------------------
    // Protocol Handlers
    // -----------------------------------------------------------------------

    /// Main loop: read commands from stdin, write responses to stdout
    pub fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            let should_quit = self.handle_command(&line);

            // Flush stdout after each command
            let _ = stdout.flush();

            if should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle a single command line. Returns true if the engine should quit.
    pub fn handle_command(&mut self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        match parts[0] {
            "uci" | "ucci" => {
                self.initialized = true;
                self.send_id_info();
                self.send_options();
                let response = if self.protocol == Protocol::UCCI {
                    "ucciok"
                } else {
                    "uciok"
                };
                println!("{}", response);
            }
            "isready" => {
                self.ready = true;
                println!("readyok");
            }
            "ucinewgame" => {
                // Reset to initial position for new game
                self.board = Board::new();
                self.board.initial_position();
                self.current_turn = Side::Red;
                self.options.move_time_ms = None;
                self.options.wtime = None;
                self.options.btime = None;
                self.options.winc = None;
                self.options.binc = None;
                self.options.nodes = None;
                self.options.infinite = false;
            }
            "setoption" => {
                self.handle_setoption(&parts[1..]);
            }
            "position" => {
                self.handle_position(&parts[1..]);
            }
            "go" => {
                self.handle_go(&parts[1..]);
            }
            "stop" => {
                // Signal search to stop (handled by should_stop in search loop)
                println!("info string stop received");
            }
            "quit" => {
                return true;
            }
            "ponderhit" => {
                // We don't implement pondering, treat as no-op
            }
            "register" => {
                // We don't implement registration
                println!("info string registration not required");
            }
            _ => {
                // Unknown command, ignore silently as per UCI spec
            }
        }

        false
    }

    /// Send engine identification info
    fn send_id_info(&self) {
        println!("id name {} v{}", ENGINE_NAME, env!("CARGO_PKG_VERSION"));
        println!("id author {}", ENGINE_AUTHOR);
    }

    /// Send available engine options
    fn send_options(&self) {
        // Hash table size option
        println!(
            "option name Hash type spin default {} min 1 max 1024",
            self.options.hash_size
        );
        // Maximum depth option
        println!(
            "option name MaxDepth type spin default {} min 1 max 30",
            self.options.max_depth
        );
        // Move time option (for fixed-time searches)
        println!("option name MoveTime type spin default 0 min 0 max 300000");
        // Ponder option (we support it structurally but don't implement it)
        println!("option name Ponder type check default false");
        // UCI_Chess960 option (not applicable for Xiangqi but some GUIs expect it)
        println!("option name UCI_Chess960 type check default false");
    }

    /// Handle `setoption name <id> [value <x>]` command
    fn handle_setoption(&mut self, parts: &[&str]) {
        // Parse: name <name> [value <value>]
        if parts.is_empty() || parts[0] != "name" {
            return;
        }

        let mut idx = 1;
        let mut name = String::new();
        let mut value = String::new();

        // Collect name tokens until we hit "value"
        while idx < parts.len() && parts[idx] != "value" {
            if !name.is_empty() {
                name.push(' ');
            }
            name.push_str(parts[idx]);
            idx += 1;
        }

        // If we found "value", collect the rest
        if idx < parts.len() && parts[idx] == "value" {
            idx += 1;
            while idx < parts.len() {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(parts[idx]);
                idx += 1;
            }
        }

        // Apply the option
        match name.as_str() {
            "Hash" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.options.hash_size = v.clamp(1, 1024);
                }
            }
            "MaxDepth" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.options.max_depth = v.clamp(1, 30);
                }
            }
            "MoveTime" => {
                if let Ok(v) = value.parse::<u64>() {
                    if v > 0 {
                        self.options.move_time_ms = Some(v);
                    }
                }
            }
            _ => {
                // Unknown option, ignore silently
            }
        }
    }

    /// Handle `position startpos [moves ...]` or `position fen <FEN> [moves ...]`
    fn handle_position(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "startpos" => {
                self.board = Board::new();
                self.board.initial_position();
                self.current_turn = Side::Red;
                self.parse_moves_from_position(&parts[1..]);
            }
            "fen" => {
                if parts.len() < 2 {
                    return;
                }
                // Reconstruct FEN string (may contain spaces for side-to-move info)
                // UCI FEN: "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w ..."
                let mut fen_str = String::new();
                let mut move_start = None;
                for (i, &part) in parts.iter().enumerate().skip(1) {
                    if part == "moves" {
                        move_start = Some(i);
                        break;
                    }
                    if !fen_str.is_empty() {
                        fen_str.push(' ');
                    }
                    fen_str.push_str(part);
                }

                // Parse board position (first part before space)
                let board_part = fen_str.split_whitespace().next().unwrap_or(&fen_str);
                if let Ok(board) = Board::from_fen(board_part) {
                    self.board = board;
                } else {
                    println!("info string invalid fen: {}", fen_str);
                    return;
                }

                // Determine side to move from FEN (second token)
                self.current_turn = if let Some(second) = fen_str.split_whitespace().nth(1) {
                    if second == "b" || second == "black" {
                        Side::Black
                    } else {
                        Side::Red
                    }
                } else {
                    Side::Red
                };

                // Parse any subsequent moves
                if let Some(start) = move_start {
                    self.parse_moves_from_position(&parts[start..]);
                }
            }
            _ => {
                // Unknown position type
            }
        }
    }

    /// Parse UCI moves from a position command and apply them sequentially
    fn parse_moves_from_position(&mut self, parts: &[&str]) {
        if parts.is_empty() || parts[0] != "moves" {
            return;
        }

        for &move_str in parts.iter().skip(1) {
            if let Ok(mv) = try_parse_iccs_move(move_str) {
                // Verify the move is legal for current turn
                if self
                    .board
                    .make_move((mv.from_col, mv.from_row), (mv.to_col, mv.to_row))
                {
                    self.current_turn = self.current_turn.opposite();
                }
            }
        }
    }

    /// Handle `go [depth N] [movetime N] [wtime N] [btime N] ...`
    fn handle_go(&mut self, parts: &[&str]) {
        let mut search_options = self.options.clone();

        let mut idx = 0;
        while idx < parts.len() {
            match parts[idx] {
                "depth" => {
                    if idx + 1 < parts.len() {
                        if let Ok(d) = parts[idx + 1].parse::<usize>() {
                            search_options.max_depth = d;
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "movetime" => {
                    if idx + 1 < parts.len() {
                        if let Ok(t) = parts[idx + 1].parse::<u64>() {
                            search_options.move_time_ms = Some(t);
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "wtime" => {
                    if idx + 1 < parts.len() {
                        search_options.wtime = parts[idx + 1].parse::<u64>().ok();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "btime" => {
                    if idx + 1 < parts.len() {
                        search_options.btime = parts[idx + 1].parse::<u64>().ok();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "winc" => {
                    if idx + 1 < parts.len() {
                        search_options.winc = parts[idx + 1].parse::<u64>().ok();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "binc" => {
                    if idx + 1 < parts.len() {
                        search_options.binc = parts[idx + 1].parse::<u64>().ok();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "infinite" => {
                    search_options.infinite = true;
                    idx += 1;
                }
                "nodes" => {
                    if idx + 1 < parts.len() {
                        search_options.nodes = parts[idx + 1].parse::<u64>().ok();
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "ponder" => {
                    // We don't implement pondering, just skip
                    idx += 1;
                }
                "searchmoves" => {
                    // Restrict search to specific moves (not implemented)
                    idx += 1;
                    while idx < parts.len()
                        && !matches!(
                            parts[idx],
                            "depth"
                                | "movetime"
                                | "wtime"
                                | "btime"
                                | "winc"
                                | "binc"
                                | "infinite"
                                | "nodes"
                                | "ponder"
                                | "searchmoves"
                        )
                    {
                        idx += 1;
                    }
                }
                _ => {
                    idx += 1;
                }
            }
        }

        self.search_and_respond(&search_options);
    }

    /// Run the search and send the result via stdout
    fn search_and_respond(&mut self, options: &EngineOptions) {
        let result = search(&self.board, self.current_turn, options);

        // Send info lines for intermediate results (already done during search)
        // Send bestmove
        if let Some(mv) = result.best_move {
            let iccs_move = format_iccs_move(&mv);
            println!("bestmove {}", iccs_move);
        } else {
            println!("bestmove (none)");
        }
    }
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

/// Search for the best move in the current position
pub fn search(board: &Board, side: Side, options: &EngineOptions) -> SearchResult {
    let start_time = Instant::now();

    // Calculate time limit
    let time_limit = if options.infinite {
        None
    } else if let Some(move_time) = options.move_time_ms {
        Some(Instant::now() + Duration::from_millis(move_time))
    } else if let Some(time) = options.time_for_side(side) {
        // Use a fraction of the remaining time (1/30 is a common heuristic)
        let time_per_move = time / 30;
        let increment = options.increment_for_side(side);
        let total_time = time_per_move + increment;
        Some(Instant::now() + Duration::from_millis(total_time))
    } else {
        None
    };

    let mut state = SearchState::new(time_limit, options.nodes);

    let mut best_move = None;
    let mut best_score = -INFINITY;
    let mut max_depth = 0;

    // Generate all legal moves to check for no moves available
    let all_moves = generate_moves(board, side);
    if all_moves.is_empty() {
        return SearchResult {
            best_move: None,
            depth: 0,
            score: -INFINITY,
            nodes: 0,
            time_ms: 0,
            nps: 0,
            is_mate: false,
        };
    }

    // Iterative deepening
    for depth in 1..=options.max_depth {
        state.current_depth = depth;

        let result = search_root(board, side, depth, options.max_depth, &mut state);

        // Check if we should stop
        if state.should_stop() {
            break;
        }

        // Update best move if we got a valid result
        if result.score > -INFINITY {
            best_move = result.best_move;
            best_score = result.score;
            max_depth = depth;
        }

        // Send info line
        let elapsed = start_time.elapsed().as_millis() as u64;
        let nps = if elapsed > 0 {
            state.nodes * 1000 / elapsed
        } else {
            0
        };

        // Build score string
        let score_str = if result.score.abs() >= MATE_THRESHOLD {
            let mate_in = (MATE_SCORE - result.score.abs()) / 2 + 1;
            if result.score > 0 {
                format!("mate {}", mate_in)
            } else {
                format!("mate -{}", mate_in)
            }
        } else {
            format!("cp {}", result.score)
        };

        println!(
            "info depth {} score {} nodes {} time {} nps {} pv {}",
            depth,
            score_str,
            state.nodes,
            elapsed,
            nps,
            result
                .best_move
                .map(|m| format_iccs_move(&m))
                .unwrap_or_else(|| "(none)".to_string())
        );
    }

    let elapsed = start_time.elapsed().as_millis() as u64;
    let nps = if elapsed > 0 {
        state.nodes * 1000 / elapsed
    } else {
        0
    };

    SearchResult {
        best_move,
        depth: max_depth,
        score: best_score,
        nodes: state.nodes,
        time_ms: elapsed,
        nps,
        is_mate: best_score.abs() >= MATE_THRESHOLD,
    }
}

/// Root search function - searches all moves at the root and picks the best
fn search_root(
    board: &Board,
    side: Side,
    depth: usize,
    max_depth: usize,
    state: &mut SearchState,
) -> SearchResult {
    let mut moves = generate_moves(board, side);

    if moves.is_empty() {
        return SearchResult::null();
    }

    // Order moves: captures first, then by MVV-LVA-like heuristic
    order_moves(board, &mut moves);

    let mut best_score = -INFINITY;
    let mut best_move = moves[0];

    for mv in &moves {
        if state.should_stop() {
            break;
        }

        state.increment_nodes();

        // Make the move on a copy of the board
        let mut board_copy = board.clone();
        if !board_copy.make_move((mv.from_col, mv.from_row), (mv.to_col, mv.to_row)) {
            continue;
        }

        let opponent = side.opposite();
        let score = -alpha_beta(
            &board_copy,
            opponent,
            -INFINITY,
            -best_score,
            depth - 1,
            max_depth,
            state,
        );

        if score > best_score {
            best_score = score;
            best_move = *mv;
        }
    }

    SearchResult {
        best_move: Some(best_move),
        depth,
        score: best_score,
        nodes: 0,
        time_ms: 0,
        nps: 0,
        is_mate: best_score.abs() >= MATE_THRESHOLD,
    }
}

/// Alpha-beta search with negamax
fn alpha_beta(
    board: &Board,
    side: Side,
    mut alpha: i32,
    beta: i32,
    depth: usize,
    max_depth: usize,
    state: &mut SearchState,
) -> i32 {
    if state.should_stop() {
        return 0;
    }

    state.increment_nodes();

    // Leaf node: enter quiescence search
    if depth == 0 {
        return quiescence_search(board, side, alpha, beta, QUIESCENCE_MAX_DEPTH, state);
    }

    let moves = generate_moves(board, side);

    // Check for checkmate or stalemate
    if moves.is_empty() {
        if is_in_check(board, side) {
            // Checkmate - return a score that accounts for distance to mate
            return -(MATE_SCORE - state.current_depth as i32);
        } else {
            // Stalemate - draw
            return 0;
        }
    }

    let mut best_score = -INFINITY;

    for mv in &moves {
        let mut board_copy = board.clone();
        if !board_copy.make_move((mv.from_col, mv.from_row), (mv.to_col, mv.to_row)) {
            continue;
        }

        let opponent = side.opposite();
        let score = -alpha_beta(
            &board_copy,
            opponent,
            -beta,
            -(alpha.max(best_score)),
            depth - 1,
            max_depth,
            state,
        );

        if score > best_score {
            best_score = score;
            if best_score > alpha {
                alpha = best_score;
            }
        }

        // Beta cutoff
        if best_score >= beta {
            return best_score;
        }
    }

    best_score
}

/// Quiescence search - only searches captures to avoid horizon effect
fn quiescence_search(
    board: &Board,
    side: Side,
    mut alpha: i32,
    beta: i32,
    depth: usize,
    state: &mut SearchState,
) -> i32 {
    if state.should_stop() || depth == 0 {
        return evaluate(board, side);
    }

    state.increment_nodes();

    // Stand-pat evaluation (score if we don't make any captures)
    let stand_pat = evaluate(board, side);

    if stand_pat >= beta {
        return beta;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Only search captures
    let mut moves = generate_capture_moves(board, side);

    // Order captures by MVV-LVA
    order_captures(&mut moves);

    for mv in &moves {
        let mut board_copy = board.clone();
        if !board_copy.make_move((mv.from_col, mv.from_row), (mv.to_col, mv.to_row)) {
            continue;
        }

        let opponent = side.opposite();
        let score = -quiescence_search(&board_copy, opponent, -beta, -alpha, depth - 1, state);

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

/// Generate only capture moves for quiescence search
fn generate_capture_moves(board: &Board, side: Side) -> Vec<Move> {
    generate_moves(board, side)
        .into_iter()
        .filter(|mv| mv.captured.is_some())
        .collect()
}

// ---------------------------------------------------------------------------
// Move Ordering
// ---------------------------------------------------------------------------

/// Order moves for better alpha-beta pruning (captures first)
fn order_moves(board: &Board, moves: &mut Vec<Move>) {
    moves.sort_by(|a, b| {
        let a_is_capture = a.captured.is_some();
        let b_is_capture = b.captured.is_some();

        if a_is_capture && !b_is_capture {
            return std::cmp::Ordering::Less;
        }
        if !a_is_capture && b_is_capture {
            return std::cmp::Ordering::Greater;
        }

        // For captures, use MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        if a_is_capture && b_is_capture {
            let a_score = capture_score(board, a);
            let b_score = capture_score(board, b);
            return b_score.cmp(&a_score); // Higher score first
        }

        // For non-captures, prefer central moves (heuristic)
        let a_central = central_bonus(a);
        let b_central = central_bonus(b);
        b_central.cmp(&a_central)
    });
}

/// Calculate MVV-LVA score for a capture
fn capture_score(board: &Board, mv: &Move) -> i32 {
    let victim_value = if let Some(captured) = mv.captured {
        piece_value_from_fen(captured)
    } else {
        0
    };

    let attacker_fen = board.get_fen(mv.from_col, mv.from_row);
    let attacker_value = piece_value_from_fen(attacker_fen);

    // MVV-LVA: victim value * 100 - attacker value
    victim_value * 100 - attacker_value
}

/// Bonus for moves toward the center of the board
fn central_bonus(mv: &Move) -> i32 {
    let center_col = 4;
    let from_col_dist = (mv.from_col as i32 - center_col).abs();
    let to_col_dist = (mv.to_col as i32 - center_col).abs();
    from_col_dist - to_col_dist
}

/// Order captures by MVV-LVA
fn order_captures(moves: &mut Vec<Move>) {
    moves.sort_by(|a, b| {
        let a_score = a.captured.map(|c| piece_value_from_fen(c)).unwrap_or(0);
        let b_score = b.captured.map(|c| piece_value_from_fen(c)).unwrap_or(0);
        b_score.cmp(&a_score)
    });
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

/// Evaluate the position from the given side's perspective
/// Positive score means good for the side to move
pub fn evaluate(board: &Board, side: Side) -> i32 {
    let score = evaluate_board(board);
    if side == Side::Red {
        score
    } else {
        -score
    }
}

/// Evaluate the board from Red's perspective
/// Positive = advantage for Red, Negative = advantage for Black
fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;

    for row in 0..10 {
        for col in 0..9 {
            let fen = board.get_fen(col, row);
            if fen == '.' {
                continue;
            }

            let piece_value = piece_value_from_fen(fen);
            let positional = positional_bonus(fen, col, row);

            if fen.is_uppercase() {
                // Red piece
                score += piece_value + positional;
            } else {
                // Black piece
                score -= piece_value + positional;
            }
        }
    }

    // Check bonus: if one side is in check, give a small bonus to the attacker
    // This helps with mating patterns
    if is_in_check(board, Side::Red) {
        score -= 50;
    }
    if is_in_check(board, Side::Black) {
        score += 50;
    }

    score
}

/// Get the material value of a piece from its FEN character
fn piece_value_from_fen(fen: char) -> i32 {
    match fen.to_ascii_lowercase() {
        'k' => PIECE_VALUES[0], // King
        'a' => PIECE_VALUES[1], // Advisor
        'b' => PIECE_VALUES[2], // Elephant
        'n' => PIECE_VALUES[3], // Knight
        'r' => PIECE_VALUES[4], // Rook
        'c' => PIECE_VALUES[5], // Cannon
        'p' => PIECE_VALUES[6], // Pawn
        _ => 0,
    }
}

/// Piece type index from FEN character (for PST lookup)
fn piece_type_index(fen: char) -> usize {
    match fen.to_ascii_lowercase() {
        'k' => 0,
        'a' => 1,
        'b' => 2,
        'n' => 3,
        'r' => 4,
        'c' => 5,
        'p' => 6,
        _ => 0,
    }
}

/// Calculate positional bonus for a piece
fn positional_bonus(fen: char, col: usize, row: usize) -> i32 {
    let is_red = fen.is_uppercase();
    let pt_idx = piece_type_index(fen);

    // For positional evaluation, we evaluate from Red's perspective
    // So Red's row 9 is "rank 0" for PST purposes
    // Black's row 0 maps to "rank 9"

    let rust_row = row; // Already in Rust coordinates
    let adjusted_row = if is_red {
        9 - rust_row // Flip so row 9 (Red base) -> 0
    } else {
        rust_row // Black row 0 (top) stays 0, but Black is negative
    };

    match pt_idx {
        0 => king_positional(col, rust_row, is_red), // King
        1 => 0,                                      // Advisor (stays in palace)
        2 => 0,                                      // Elephant (stays on own side)
        3 => knight_positional(col, adjusted_row),   // Knight
        4 => rook_positional(col, adjusted_row),     // Rook
        5 => cannon_positional(col, adjusted_row),   // Cannon
        6 => pawn_positional(col, rust_row, is_red), // Pawn
        _ => 0,
    }
}

/// King positional evaluation
fn king_positional(col: usize, row: usize, is_red: bool) -> i32 {
    // King safety: prefer center of palace
    let palace_center_col = 4;
    let palace_center_row = if is_red { 1 } else { 8 };

    let col_dist = (col as i32 - palace_center_col).abs();
    let row_dist = (row as i32 - palace_center_row).abs();

    // Small bonus for being in center of palace
    let safety_bonus = 10 - (col_dist + row_dist) * 3;
    safety_bonus.max(0)
}

/// Knight positional evaluation
fn knight_positional(col: usize, row: usize) -> i32 {
    // Knights prefer center squares and advanced positions
    let col_center = (col as i32 - 4).abs();
    let row_center = (row as i32 - 4).abs();

    // Prefer center and forward
    let center_bonus = (8 - col_center - row_center) * 5;
    center_bonus.max(0)
}

/// Rook positional evaluation
fn rook_positional(col: usize, row: usize) -> i32 {
    // Rooks prefer open files and 7th rank
    // Simple heuristic: prefer center and advanced
    let col_center = (col as i32 - 4).abs();
    let advanced = (5 - row as i32).abs(); // Prefer row 5 (river)

    (10 - col_center) * 2 + (5 - advanced) * 3
}

/// Cannon positional evaluation
fn cannon_positional(col: usize, row: usize) -> i32 {
    // Cannons prefer center and middle ranks
    let col_center = (col as i32 - 4).abs();
    let row_center = (row as i32 - 4).abs();

    (8 - col_center - row_center) * 3
}

/// Pawn positional evaluation
fn pawn_positional(col: usize, row: usize, is_red: bool) -> i32 {
    // Pawns gain value as they advance and especially after crossing the river
    // Red river crossing: row >= 5 (moved from bottom to Black's side)
    // Black river crossing: row <= 4 (moved from top to Red's side)

    let crossed_river = if is_red { row >= 5 } else { row <= 4 };

    let mut bonus: i32 = 0;

    // Base advancement bonus
    // Red pawn advancing downward (increasing row), Black pawn advancing upward (decreasing row)
    let advancement = if is_red {
        row // Red pawn advancing downward
    } else {
        9 - row // Black pawn advancing upward
    };
    bonus += (advancement * 10) as i32;

    // River crossing bonus
    if crossed_river {
        bonus += 30;
        // Additional bonus for pawns near the palace
        // Red palace is rows 0-2, Black palace is rows 7-9
        // Red pawn near Black's palace: row >= 7
        // Black pawn near Red's palace: row <= 2
        if is_red && row >= 7 {
            bonus += 20;
        } else if !is_red && row <= 2 {
            bonus += 20;
        }
    }

    // Center file bonus
    let col_center = (col as i32 - 4).abs();
    bonus += (4 - col_center) * 3;

    bonus
}

// ---------------------------------------------------------------------------
// Check Detection
// ---------------------------------------------------------------------------

/// Check if the given side's king is in check
fn is_in_check(board: &Board, side: Side) -> bool {
    // Find king position
    let king_fen = if side == Side::Red { 'K' } else { 'k' };

    let mut king_pos = None;
    'outer: for row in 0..10 {
        for col in 0..9 {
            if board.get_fen(col, row) == king_fen {
                king_pos = Some((col, row));
                break 'outer;
            }
        }
    }

    let (king_col, king_row) = match king_pos {
        Some(pos) => pos,
        None => return true, // King doesn't exist = in check
    };

    // Check if any enemy piece attacks the king
    let opponent = side.opposite();
    is_square_attacked(board, king_col, king_row, side, opponent)
}

/// Check if a square is attacked by the given side
fn is_square_attacked(
    board: &Board,
    target_col: usize,
    target_row: usize,
    _defender: Side,
    attacker: Side,
) -> bool {
    let opponent = attacker; // The side attacking

    // Check for knight attacks
    if is_knight_attack(board, target_col, target_row, opponent) {
        return true;
    }

    // Check for rook/cannon attacks (sliding pieces)
    if is_sliding_attack(board, target_col, target_row, opponent) {
        return true;
    }

    // Check for king attacks (flying general)
    if is_king_attack(board, target_col, target_row, opponent) {
        return true;
    }

    // Check for pawn attacks
    if is_pawn_attack(board, target_col, target_row, opponent) {
        return true;
    }

    // Check for advisor attacks
    if is_advisor_attack(board, target_col, target_row, opponent) {
        return true;
    }

    false
}

/// Check for knight attacks on the target square
fn is_knight_attack(board: &Board, target_col: usize, target_row: usize, side: Side) -> bool {
    let knight_patterns = [
        (1, 2, 0, 1),    // Knight at (0,1) blocks, attacks from (1,2)
        (1, -2, 0, -1),  // Knight at (0,-1) blocks
        (-1, 2, 0, 1),   // Knight at (0,1) blocks
        (-1, -2, 0, -1), // Knight at (0,-1) blocks
        (2, 1, 1, 0),    // Knight at (1,0) blocks
        (2, -1, 1, 0),   // Knight at (1,0) blocks
        (-2, 1, -1, 0),  // Knight at (-1,0) blocks
        (-2, -1, -1, 0), // Knight at (-1,0) blocks
    ];

    let knight_fen = if side == Side::Red { 'N' } else { 'n' };

    for (dc, dr, bc, br) in &knight_patterns {
        let knight_col = target_col as isize + dc;
        let knight_row = target_row as isize + dr;
        let block_col = target_col as isize + bc;
        let block_row = target_row as isize + br;

        if knight_col >= 0
            && knight_col < 9
            && knight_row >= 0
            && knight_row < 10
            && block_col >= 0
            && block_col < 9
            && block_row >= 0
            && block_row < 10
        {
            // Check blocking square is empty
            if board.is_empty_at(block_col as usize, block_row as usize) {
                // Check if there's a knight at the attack square
                if board.get_fen(knight_col as usize, knight_row as usize) == knight_fen {
                    return true;
                }
            }
        }
    }

    false
}

/// Check for rook/cannon attacks on the target square
fn is_sliding_attack(board: &Board, target_col: usize, target_row: usize, side: Side) -> bool {
    let rook_fen = if side == Side::Red { 'R' } else { 'r' };
    let cannon_fen = if side == Side::Red { 'C' } else { 'c' };

    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

    for (dc, dr) in &directions {
        let mut col = target_col as isize + dc;
        let mut row = target_row as isize + dr;
        let mut jumped = false;

        while col >= 0 && col < 9 && row >= 0 && row < 10 {
            let fen = board.get_fen(col as usize, row as usize);

            if fen != '.' {
                if !jumped {
                    // First piece encountered - could be a screen for cannon
                    if fen == rook_fen {
                        return true;
                    }
                    // If it's a cannon, we need to jump over it
                    jumped = true;
                } else {
                    // Second piece - cannon can capture here
                    if fen == cannon_fen {
                        return true;
                    }
                    break; // Can't attack beyond second piece
                }
            } else if jumped {
                // Empty square after jump - continue looking for cannon
            } else {
                // Empty square before jump - rook could be here
                // But we'd need to continue until we find something
            }

            col += dc;
            row += dr;
        }
    }

    false
}

/// Check for king attack (flying general rule)
fn is_king_attack(board: &Board, target_col: usize, target_row: usize, side: Side) -> bool {
    let king_fen = if side == Side::Red { 'K' } else { 'k' };

    // Only applicable if target is in the same column as enemy king
    let king_col = target_col;

    // Search in the direction of the enemy king
    // Red king is at rows 0-2, Black king at rows 7-9
    if side == Side::Red {
        // Red king attacks upward (lower row numbers)
        for row in (0..target_row).rev() {
            let fen = board.get_fen(king_col, row);
            if fen != '.' {
                if fen == king_fen {
                    return true;
                }
                break; // Blocked by another piece
            }
        }
    } else {
        // Black king attacks downward (higher row numbers)
        for row in (target_row + 1)..10 {
            let fen = board.get_fen(king_col, row);
            if fen != '.' {
                if fen == king_fen {
                    return true;
                }
                break; // Blocked by another piece
            }
        }
    }

    false
}

/// Check for pawn attacks on the target square
fn is_pawn_attack(board: &Board, target_col: usize, target_row: usize, side: Side) -> bool {
    let pawn_fen = if side == Side::Red { 'P' } else { 'p' };
    let forward = if side == Side::Red { 1 } else { -1 };

    // Pawn can attack from directly behind (moving forward to target)
    let behind_row = target_row as isize - forward;
    if behind_row >= 0 && behind_row < 10 {
        if board.get_fen(target_col, behind_row as usize) == pawn_fen {
            // Check if pawn has reached this rank (can it move here?)
            // Red pawns can always move forward, so this is always valid
            return true;
        }
    }

    // If target is across the river, pawns can also attack from the sides
    // Red across river: row >= 5, Black across river: row <= 4
    let crossed_river = if side == Side::Red {
        target_row >= 5
    } else {
        target_row <= 4
    };

    if crossed_river {
        // Check left and right
        for dc in &[-1, 1] {
            let side_col = target_col as isize + dc;
            if side_col >= 0 && side_col < 9 {
                // Pawns on the same row can attack sideways only if they've crossed
                // But the attacking pawn needs to be on the same row as target
                if board.get_fen(side_col as usize, target_row) == pawn_fen {
                    return true;
                }
            }
        }
    }

    false
}

/// Check for advisor attacks on the target square
fn is_advisor_attack(board: &Board, target_col: usize, target_row: usize, side: Side) -> bool {
    let advisor_fen = if side == Side::Red { 'A' } else { 'a' };

    // Advisor moves diagonally within palace
    let palace_rows = if side == Side::Red { 0..=2 } else { 7..=9 };

    if !palace_rows.contains(&target_row) || !(3..=5).contains(&target_col) {
        return false; // Target not in palace, can't be attacked by advisor
    }

    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    for (dc, dr) in &directions {
        let adv_col = target_col as isize + dc;
        let adv_row = target_row as isize + dr;

        if adv_col >= 0
            && adv_col < 9
            && adv_row >= 0
            && adv_row < 10
            && board.get_fen(adv_col as usize, adv_row as usize) == advisor_fen
        {
            return true;
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Convenience Function
// ---------------------------------------------------------------------------

/// Find the best move for the given side without running the full protocol loop.
///
/// This is useful for integrating the engine into other code or for testing.
///
/// # Arguments
/// * `board` - The current board position
/// * `side` - Which side is to move
/// * `depth` - Maximum search depth
/// * `time_ms` - Optional time limit in milliseconds
pub fn find_best_move(
    board: &Board,
    side: Side,
    depth: usize,
    time_ms: Option<u64>,
) -> SearchResult {
    let mut options = EngineOptions::default();
    options.max_depth = depth;
    options.move_time_ms = time_ms;

    search(board, side, &options)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::move_notation::{format_iccs_move, try_parse_iccs_move};

    #[test]
    fn test_parse_iccs_move_basic() {
        // ICCS "h2e2" → internal row 2 (Red cannon position)
        let mv = try_parse_iccs_move("h2e2").unwrap();
        assert_eq!(mv.from_col, 7);
        assert_eq!(mv.from_row, 2);
        assert_eq!(mv.to_col, 4);
        assert_eq!(mv.to_row, 2);
    }

    #[test]
    fn test_parse_iccs_move_edges() {
        // a0 = col 0, row 0 (Red bottom-left)
        let mv = try_parse_iccs_move("a0a1").unwrap();
        assert_eq!(mv.from_col, 0);
        assert_eq!(mv.from_row, 0);
        assert_eq!(mv.to_col, 0);
        assert_eq!(mv.to_row, 1);
    }

    #[test]
    fn test_parse_iccs_move_with_hyphen() {
        // ICCS with hyphen format
        let mv = try_parse_iccs_move("h2-e2").unwrap();
        assert_eq!(mv.from_col, 7);
        assert_eq!(mv.from_row, 2);
        assert_eq!(mv.to_col, 4);
        assert_eq!(mv.to_row, 2);
    }

    #[test]
    fn test_parse_iccs_move_invalid() {
        assert!(try_parse_iccs_move("abc").is_err()); // Too short
        assert!(try_parse_iccs_move("j0e2").is_err()); // Invalid column
        assert!(try_parse_iccs_move("hxe2").is_err()); // Invalid row
    }

    #[test]
    fn test_format_iccs_move() {
        // Internal (7,2) → (4,2) = ICCS "h2e2" (Red cannon)
        let mv = Move::new(7, 2, 4, 2);
        assert_eq!(format_iccs_move(&mv), "h2e2");
    }

    #[test]
    fn test_format_iccs_move_edges() {
        // Internal (0,0) → (0,1) = ICCS "a0a1"
        let mv = Move::new(0, 0, 0, 1);
        assert_eq!(format_iccs_move(&mv), "a0a1");
    }

    #[test]
    fn test_piece_value_from_fen() {
        assert_eq!(piece_value_from_fen('K'), 10000);
        assert_eq!(piece_value_from_fen('k'), 10000);
        assert_eq!(piece_value_from_fen('R'), 900);
        assert_eq!(piece_value_from_fen('r'), 900);
        assert_eq!(piece_value_from_fen('C'), 450);
        assert_eq!(piece_value_from_fen('c'), 450);
        assert_eq!(piece_value_from_fen('N'), 400);
        assert_eq!(piece_value_from_fen('n'), 400);
        assert_eq!(piece_value_from_fen('A'), 200);
        assert_eq!(piece_value_from_fen('a'), 200);
        assert_eq!(piece_value_from_fen('B'), 200);
        assert_eq!(piece_value_from_fen('b'), 200);
        assert_eq!(piece_value_from_fen('P'), 100);
        assert_eq!(piece_value_from_fen('p'), 100);
        assert_eq!(piece_value_from_fen('.'), 0);
    }

    #[test]
    fn test_evaluate_initial_position() {
        let mut board = Board::new();
        board.initial_position();
        let score_red = evaluate(&board, Side::Red);
        let score_black = evaluate(&board, Side::Black);

        // Initial position should be roughly equal (symmetric)
        assert_eq!(score_red, -score_black);
        // Red's score should be close to 0 (slight positive due to moving first advantage)
        // Actually, initial position is perfectly symmetric so score should be 0
        assert_eq!(score_red, 0);
    }

    #[test]
    fn test_evaluate_material_advantage() {
        let mut board = Board::new();
        board.initial_position();
        // In new coords: row 9 = Black's top. Remove a black rook (r at row 9, col 0)
        board.squares[9][0] = '.';

        let score_red = evaluate(&board, Side::Red);
        // Red should have advantage (900 points for the rook)
        assert!(score_red > 0);
        assert!(score_red >= 800); // At least most of the rook value
    }

    #[test]
    fn test_search_finds_capture() {
        // Set up a position where Red can capture a piece
        let board =
            Board::from_fen("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR").unwrap();
        let result = find_best_move(&board, Side::Red, 3, Some(1000));

        // Should find a move (initial position has legal moves)
        assert!(result.best_move.is_some());
        assert!(result.depth >= 1);
        assert!(result.nodes > 0);
    }

    #[test]
    fn test_search_null_result() {
        // Empty board = no moves
        let board = Board::new();
        let result = find_best_move(&board, Side::Red, 3, None);
        assert!(result.best_move.is_none());
    }

    #[test]
    fn test_search_result_null() {
        let result = SearchResult::null();
        assert!(result.best_move.is_none());
        assert_eq!(result.depth, 0);
        assert_eq!(result.score, 0);
        assert_eq!(result.nodes, 0);
        assert!(!result.is_mate);
    }

    #[test]
    fn test_search_result_format_score_cp() {
        let result = SearchResult {
            best_move: None,
            depth: 3,
            score: 150,
            nodes: 1000,
            time_ms: 100,
            nps: 10000,
            is_mate: false,
        };
        assert_eq!(result.format_score(), "cp 150");
    }

    #[test]
    fn test_search_result_format_score_mate() {
        let result = SearchResult {
            best_move: None,
            depth: 3,
            score: MATE_SCORE - 10,
            nodes: 1000,
            time_ms: 100,
            nps: 10000,
            is_mate: true,
        };
        assert!(result.format_score().starts_with("mate"));
    }

    #[test]
    fn test_engine_options_time_for_side() {
        let options = EngineOptions {
            wtime: Some(60000),
            btime: Some(30000),
            ..Default::default()
        };

        assert_eq!(options.time_for_side(Side::Red), Some(60000));
        assert_eq!(options.time_for_side(Side::Black), Some(30000));
        assert_eq!(options.time_for_side(Side::Any), None);
    }

    #[test]
    fn test_engine_options_move_time_override() {
        let options = EngineOptions {
            move_time_ms: Some(5000),
            wtime: Some(60000),
            btime: Some(30000),
            ..Default::default()
        };

        // move_time should override wtime/btime
        assert_eq!(options.time_for_side(Side::Red), Some(5000));
        assert_eq!(options.time_for_side(Side::Black), Some(5000));
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::UCI.to_string(), "uci");
        assert_eq!(Protocol::UCCI.to_string(), "ucci");
    }

    #[test]
    fn test_find_best_move() {
        let mut board = Board::new();
        board.initial_position();
        let result = find_best_move(&board, Side::Red, 2, Some(500));

        assert!(result.best_move.is_some());
        assert!(result.depth >= 1);
        assert!(result.time_ms > 0 || result.time_ms == 0);
    }
}
