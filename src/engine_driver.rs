//! Async engine driver for UCI/UCCI engines.
//!
//! This module provides a tokio-based async driver for communicating with
//! external UCI/UCCI engine processes (e.g., Pikafish, EleEye). It handles
//! process lifecycle, protocol handshakes, command sending, and output parsing.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────┐    send()     ┌──────────────┐    stdin    ┌──────────┐
//! │  Client  │ ───────────► │  EngineDriver │ ─────────► │  Engine  │
//! │          │ ◄─────────── │  (bg task)    │ ◄───────── │  Process │
//! └──────────┘    recv()     └──────────────┘    stdout   └──────────┘
//!                               │
//!                               ▼ event_queue
//!                         EngineEvent
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use cchess_rs::engine_driver::*;
//!
//! # async fn example() -> Result<(), String> {
//! let path = resolve_engine_path("PIKAFISH_PATH", "engine/pikafish/pikafish.exe");
//! let mut engine = EngineDriver::spawn(path, Protocol::Uci).await?;
//! engine.init().await?;
//! engine.position_fen(INITIAL_FEN).await?;
//! let result = engine.search_movetime(1000).await?;
//! println!("Best move: {:?}", result.bestmove);
//! engine.quit().await;
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};

// ---------------------------------------------------------------------------
// Shared Types
// ---------------------------------------------------------------------------

/// Score type - either centipawns or mate distance.
#[derive(Debug, Clone, PartialEq)]
pub enum Score {
    /// Centipawn score (positive = advantage for side to move)
    Cp(i64),
    /// Mate in N moves (positive = winning for side to move)
    Mate(i32),
}

impl Score {
    /// Get the centipawn value if available.
    pub fn cp(&self) -> Option<i64> {
        match self {
            Score::Cp(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the mate distance if available.
    pub fn mate(&self) -> Option<i32> {
        match self {
            Score::Mate(v) => Some(*v),
            _ => None,
        }
    }

    /// Check if this score represents a forced mate.
    pub fn is_mate(&self) -> bool {
        matches!(self, Score::Mate(_))
    }
}

/// Parsed search information from engine `info` lines.
#[derive(Debug, Clone, Default)]
pub struct SearchInfo {
    /// Search depth (plies)
    pub depth: u32,
    /// Selective depth (plies)
    pub seldepth: Option<u32>,
    /// Time used (milliseconds)
    pub time_ms: Option<u64>,
    /// Nodes searched
    pub nodes: Option<u64>,
    /// Nodes per second
    pub nps: Option<u64>,
    /// Hash table usage (per mille, 0-1000)
    pub hashfull: Option<u32>,
    /// Number of PV lines (for multi-PV mode)
    pub multipv: Option<u32>,
    /// Score (cp or mate)
    pub score: Option<Score>,
    /// Current move being searched
    pub currmove: Option<String>,
    /// Move number being searched
    pub currmovenumber: Option<u32>,
    /// Principal variation (best line found)
    pub pv: Vec<String>,
    /// Number of root moves (for root move ordering info)
    pub root_moves: Option<u32>,
}

/// Engine option description (from `option` lines during init).
#[derive(Debug, Clone)]
pub struct EngineOption {
    /// Option name (e.g., "Hash", "Threads")
    pub name: String,
    /// Option type (e.g., "spin", "check", "combo", "string", "button")
    pub r#type: String,
    /// Default value
    pub default: Option<String>,
    /// Minimum value (for spin type)
    pub min: Option<i64>,
    /// Maximum value (for spin type)
    pub max: Option<i64>,
    /// Valid values (for combo type)
    pub var_values: Vec<String>,
}

/// Events emitted by the engine driver.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// Engine process started
    Started,
    /// Engine identification: `id name <name>` or `id author <author>`
    Id {
        name: Option<String>,
        author: Option<String>,
    },
    /// Engine option available: `option name <name> type <type> ...`
    Option(EngineOption),
    /// Engine ready: `uciok`, `ucciok`, or `readyok`
    Ready,
    /// Search info line: `info depth ...`
    Info(SearchInfo),
    /// Best move found: `bestmove <move> [ponder <move>]`
    BestMove {
        bestmove: String,
        ponder: Option<String>,
    },
    /// No best move: `nobestmove` (UCCI engines may return this)
    NoBestMove,
    /// Info string: `info string <text>`
    InfoString(String),
    /// Error from engine stderr
    Stderr(String),
    /// Engine process exited with the given exit code
    Exited(i32),
}

impl EngineEvent {
    /// Check if this is a Ready event.
    pub fn is_ready(&self) -> bool {
        matches!(self, EngineEvent::Ready)
    }

    /// Check if this is a BestMove event.
    pub fn is_bestmove(&self) -> bool {
        matches!(self, EngineEvent::BestMove { .. })
    }

    /// Get the bestmove string if this is a BestMove event.
    pub fn bestmove(&self) -> Option<&str> {
        match self {
            EngineEvent::BestMove { bestmove, .. } => Some(bestmove),
            _ => None,
        }
    }
}

/// Protocol type for the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// Universal Chess Interface (used by Pikafish)
    Uci,
    /// Universal Chinese Chess Interface (used by EleEye)
    Ucci,
}

// ---------------------------------------------------------------------------
// Search Result
// ---------------------------------------------------------------------------

/// Aggregated search result from the async engine driver.
#[derive(Debug, Clone)]
pub struct SearchResultAsync {
    /// Best move returned by the engine
    pub bestmove: Option<String>,
    /// Ponder move (engine's expected response)
    pub ponder: Option<String>,
    /// All info events collected during search
    pub info_events: Vec<SearchInfo>,
    /// Final (deepest) search info
    pub final_info: Option<SearchInfo>,
    /// All raw events (including non-info events like option updates)
    pub raw_events: Vec<EngineEvent>,
}

impl SearchResultAsync {
    /// Build a SearchResultAsync from a slice of EngineEvents.
    pub fn from_events(events: &[EngineEvent]) -> Self {
        let mut bestmove = None;
        let mut ponder = None;
        let mut info_events = Vec::new();

        for event in events {
            match event {
                EngineEvent::BestMove {
                    bestmove: mv,
                    ponder: p,
                } => {
                    bestmove = Some(mv.clone());
                    ponder = p.clone();
                }
                EngineEvent::Info(info) => {
                    info_events.push(info.clone());
                }
                _ => {}
            }
        }

        let final_info = info_events.last().cloned();

        Self {
            bestmove,
            ponder,
            info_events,
            final_info,
            raw_events: events.to_vec(),
        }
    }

    /// Get the max depth reached.
    pub fn depth(&self) -> Option<u32> {
        self.final_info.as_ref().map(|i| i.depth)
    }

    /// Get the nodes searched from the final info.
    pub fn nodes(&self) -> Option<u64> {
        self.final_info.as_ref().and_then(|i| i.nodes)
    }

    /// Get the nodes per second from the final info.
    pub fn nps(&self) -> Option<u64> {
        self.final_info.as_ref().and_then(|i| i.nps)
    }

    /// Get the search time in ms from the final info.
    pub fn time_ms(&self) -> Option<u64> {
        self.final_info.as_ref().and_then(|i| i.time_ms)
    }

    /// Get the score from the final info.
    pub fn score(&self) -> Option<&Score> {
        self.final_info.as_ref().and_then(|i| i.score.as_ref())
    }

    /// Get the principal variation as a space-separated string.
    pub fn pv_string(&self) -> String {
        self.final_info
            .as_ref()
            .map(|i| i.pv.join(" "))
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Engine Driver
// ---------------------------------------------------------------------------

/// Async engine driver that manages a UCI/UCCI engine process.
///
/// The engine runs in a background tokio task. Commands are sent via
/// `send()` and engine output is received as `EngineEvent` via `recv()`.
pub struct EngineDriver {
    /// Channel to send commands to the engine
    cmd_tx: mpsc::UnboundedSender<String>,
    /// Channel to receive events from the engine (wrapped in Arc<Mutex> for shared access)
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<EngineEvent>>>,
    /// Handle to the background task
    handle: Option<tokio::task::JoinHandle<()>>,
    /// Protocol type
    protocol: Protocol,
    /// Whether the engine is ready
    ready: bool,
    /// Engine name (discovered after init)
    engine_name: Option<String>,
}

impl EngineDriver {
    /// Spawn a new engine process.
    ///
    /// The engine is started in the background. Call `init()` to complete
    /// the protocol handshake.
    pub async fn spawn(exe_path: PathBuf, protocol: Protocol) -> Result<Self, String> {
        // Normalize path separators for Windows
        let exe_str = exe_path.to_string_lossy().replace('/', "\\");
        let exe_path = PathBuf::from(&exe_str);

        if !exe_path.exists() {
            return Err(format!("engine not found: {}", exe_path.display()));
        }

        let dir = exe_path
            .parent()
            .ok_or_else(|| "engine has no parent dir".to_string())?
            .to_path_buf();

        eprintln!("[EngineDriver] Spawning {:?} in {:?}", exe_path, dir);

        let mut child = Command::new(&exe_path)
            .current_dir(&dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn {:?}: {}", exe_path, e))?;

        let stdin = child.stdin.take().ok_or("failed to capture stdin")?;
        let stdout = child.stdout.take().ok_or("failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("failed to capture stderr")?;

        // Child is dropped here, but process lives on via the pipes
        drop(child);

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<EngineEvent>();
        let event_rx = Arc::new(Mutex::new(event_rx));

        let handle = tokio::spawn(Self::event_loop(
            stdin,
            stdout,
            stderr,
            cmd_rx,
            event_tx.clone(),
        ));

        let _ = event_tx.send(EngineEvent::Started);

        // Give the engine a moment to initialize and output its greeting
        tokio::time::sleep(Duration::from_millis(200)).await;

        Ok(Self {
            cmd_tx,
            event_rx,
            handle: Some(handle),
            protocol,
            ready: false,
            engine_name: None,
        })
    }

    /// The main event loop that runs in a background task.
    ///
    /// It concurrently:
    /// - Reads lines from engine stdout and parses them into events
    /// - Reads lines from engine stderr and emits error events
    /// - Receives commands from the command channel and writes them to engine stdin
    async fn event_loop(
        mut stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        stderr: tokio::process::ChildStderr,
        mut cmd_rx: mpsc::UnboundedReceiver<String>,
        event_tx: mpsc::UnboundedSender<EngineEvent>,
    ) {
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        loop {
            tokio::select! {
                biased;
                // Read from engine stdout
                result = stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            let event = Self::parse_line(&line);
                            let _ = event_tx.send(event);
                        }
                        Ok(None) => {
                            let _ = event_tx.send(EngineEvent::Exited(0));
                            break;
                        }
                        Err(e) => {
                            let _ = event_tx.send(EngineEvent::Stderr(e.to_string()));
                            break;
                        }
                    }
                }
                // Read from engine stderr
                result = stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) if !line.is_empty() => {
                            let _ = event_tx.send(EngineEvent::Stderr(line));
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                // Send commands to engine
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(cmd) => {
                            let cmd_with_newline = format!("{}\n", cmd);
                            if let Err(e) = stdin.write_all(cmd_with_newline.as_bytes()).await {
                                let _ = event_tx.send(EngineEvent::Stderr(format!("write error: {}", e)));
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    }

    /// Parse a single line of engine output into an EngineEvent.
    pub fn parse_line(line: &str) -> EngineEvent {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return EngineEvent::InfoString(String::new());
        }

        // Protocol ready
        if trimmed == "uciok" || trimmed == "ucciok" || trimmed == "readyok" {
            return EngineEvent::Ready;
        }

        // Best move
        if let Some(rest) = trimmed.strip_prefix("bestmove ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            let bestmove = parts.first().map(|s| s.to_string()).unwrap_or_default();
            let ponder = if parts.len() >= 3 && parts[1] == "ponder" {
                Some(parts[2].to_string())
            } else {
                None
            };
            return EngineEvent::BestMove { bestmove, ponder };
        }

        // No best move
        if trimmed == "nobestmove" {
            return EngineEvent::NoBestMove;
        }

        // Engine ID
        if let Some(rest) = trimmed.strip_prefix("id ") {
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() == 2 {
                match parts[0] {
                    "name" => {
                        return EngineEvent::Id {
                            name: Some(parts[1].to_string()),
                            author: None,
                        }
                    }
                    "author" => {
                        return EngineEvent::Id {
                            name: None,
                            author: Some(parts[1].to_string()),
                        }
                    }
                    _ => {}
                }
            }
            return EngineEvent::InfoString(trimmed.to_string());
        }

        // Engine options
        if let Some(rest) = trimmed.strip_prefix("option ") {
            return EngineEvent::Option(Self::parse_option(rest));
        }

        // Search info
        if trimmed.starts_with("info ") {
            // Check if it's an info string
            if let Some(rest) = trimmed.strip_prefix("info string ") {
                return EngineEvent::InfoString(rest.to_string());
            }
            // Parse as search info
            if let Some(info) = parse_info_line(trimmed) {
                return EngineEvent::Info(info);
            }
        }

        // Unknown line - emit as info string
        EngineEvent::InfoString(trimmed.to_string())
    }

    /// Parse an `option` line into an EngineOption.
    fn parse_option(rest: &str) -> EngineOption {
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
                    // For string types, default might be multi-word
                    // For spin types, it's a single number
                    if r#type == "string" || r#type == "filename" {
                        // Find the next keyword to know where default ends
                        let mut end = parts.len();
                        for (j, &part) in parts.iter().enumerate().skip(i + 1) {
                            if matches!(part, "min" | "max" | "var") {
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
                    // Collect var values until next keyword
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

        EngineOption {
            name,
            r#type,
            default,
            min,
            max,
            var_values,
        }
    }

    /// Send a command to the engine.
    pub async fn send(&self, cmd: &str) -> Result<(), String> {
        self.cmd_tx
            .send(cmd.to_string())
            .map_err(|e| format!("send failed: {}", e))
    }

    /// Receive the next event from the engine.
    pub async fn recv(&self) -> Option<EngineEvent> {
        self.event_rx.lock().await.recv().await
    }

    /// Receive events until one matching the predicate is found.
    ///
    /// Returns all events received (including the matching one).
    pub async fn collect_until<F>(
        &self,
        predicate: F,
        timeout: Duration,
    ) -> Result<Vec<EngineEvent>, String>
    where
        F: Fn(&EngineEvent) -> bool,
    {
        let mut events = Vec::new();

        let recv_fut = async {
            loop {
                match self.event_rx.lock().await.recv().await {
                    Some(event) => {
                        events.push(event.clone());
                        if predicate(&event) {
                            return Ok(events);
                        }
                    }
                    None => return Err("engine closed".to_string()),
                }
            }
        };

        tokio::time::timeout(timeout, recv_fut)
            .await
            .map_err(|_| format!("timeout after {}ms waiting for event", timeout.as_millis()))?
    }

    /// Receive events until a bestmove is found.
    pub async fn wait_bestmove(&self, timeout: Duration) -> Result<Vec<EngineEvent>, String> {
        self.collect_until(
            |e| e.is_bestmove() || matches!(e, EngineEvent::NoBestMove),
            timeout,
        )
        .await
    }

    /// Initialize the engine protocol (uci/ucci + isready).
    pub async fn init(&mut self) -> Result<Vec<EngineEvent>, String> {
        let protocol_cmd = match self.protocol {
            Protocol::Uci => "uci",
            Protocol::Ucci => "ucci",
        };

        self.send(protocol_cmd).await?;

        let events = self
            .collect_until(|e| e.is_ready(), Duration::from_secs(10))
            .await?;

        // Extract engine name
        for event in &events {
            if let EngineEvent::Id {
                name: Some(name), ..
            } = event
            {
                self.engine_name = Some(name.clone());
            }
        }

        // Send isready
        self.send("isready").await?;

        // Wait for readyok
        let more_events = self
            .collect_until(|e| e.is_ready(), Duration::from_secs(10))
            .await?;

        self.ready = true;
        let mut all_events = events;
        all_events.extend(more_events);
        Ok(all_events)
    }

    /// Set an engine option.
    ///
    /// For UCI engines: `setoption name <name> value <value>`
    /// For UCCI engines: `setoption <name> <value>`
    pub async fn setoption(&self, name: &str, value: &str) -> Result<(), String> {
        let cmd = match self.protocol {
            Protocol::Uci => format!("setoption name {} value {}", name, value),
            Protocol::Ucci => format!("setoption {} {}", name, value),
        };
        self.send(&cmd).await
    }

    /// Set a position by FEN string.
    pub async fn position_fen(&self, fen: &str) -> Result<(), String> {
        self.send(&format!("position fen {}", fen)).await
    }

    /// Set a position with moves from startpos.
    pub async fn position_startpos_moves(&self, moves: &str) -> Result<(), String> {
        self.send(&format!("position startpos moves {}", moves))
            .await
    }

    /// Start a search with time limit.
    ///
    /// For UCI engines: `go movetime <ms>`
    /// For UCCI engines: `go time <centiseconds>`
    pub async fn go_movetime(&self, time_ms: u64) -> Result<(), String> {
        let cmd = match self.protocol {
            Protocol::Uci => format!("go movetime {}", time_ms),
            Protocol::Ucci => format!("go time {}", time_ms / 10), // UCCI uses centiseconds
        };
        self.send(&cmd).await
    }

    /// Start a search with depth limit.
    pub async fn go_depth(&self, depth: u32) -> Result<(), String> {
        self.send(&format!("go depth {}", depth)).await
    }

    /// Search and return the result (bestmove + all info events).
    pub async fn search_movetime(&self, time_ms: u64) -> Result<SearchResultAsync, String> {
        self.go_movetime(time_ms).await?;
        let events = self.wait_bestmove(Duration::from_secs(30)).await?;
        Ok(SearchResultAsync::from_events(&events))
    }

    /// Search with depth limit and return the result.
    pub async fn search_depth(&self, depth: u32) -> Result<SearchResultAsync, String> {
        self.go_depth(depth).await?;
        let events = self.wait_bestmove(Duration::from_secs(60)).await?;
        Ok(SearchResultAsync::from_events(&events))
    }

    /// Check if the engine is ready.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Get the engine name (after init).
    pub fn engine_name(&self) -> Option<&str> {
        self.engine_name.as_deref()
    }

    /// Get the protocol type.
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// Send quit and wait for the engine to exit.
    pub async fn quit(&mut self) {
        let _ = self.send("quit").await;
        // Wait for exit or timeout
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            self.collect_until(
                |e| matches!(e, EngineEvent::Exited(_)),
                Duration::from_secs(2),
            ),
        )
        .await;

        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

impl Drop for EngineDriver {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// Public Parsing Functions
// ---------------------------------------------------------------------------

/// Parse a single `info` line into a SearchInfo struct.
///
/// Handles standard UCI info fields: depth, seldepth, time, nodes, nps,
/// hashfull, multipv, score (cp/mate), currmove, currmovenumber, pv, string.
pub fn parse_info_line(line: &str) -> Option<SearchInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() || parts[0] != "info" {
        return None;
    }

    let mut info = SearchInfo::default();
    let mut i = 1; // skip "info"

    while i < parts.len() {
        match parts[i] {
            "depth" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u32>() {
                        info.depth = v;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "seldepth" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u32>() {
                        info.seldepth = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "time" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u64>() {
                        info.time_ms = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "nodes" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u64>() {
                        info.nodes = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "nps" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u64>() {
                        info.nps = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "hashfull" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u32>() {
                        info.hashfull = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "multipv" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u32>() {
                        info.multipv = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "score" => {
                if let Some(kind) = parts.get(i + 1) {
                    if let Some(val_str) = parts.get(i + 2) {
                        if *kind == "cp" {
                            if let Ok(v) = val_str.parse::<i64>() {
                                info.score = Some(Score::Cp(v));
                            }
                            i += 3;
                        } else if *kind == "mate" {
                            if let Ok(v) = val_str.parse::<i32>() {
                                info.score = Some(Score::Mate(v));
                            }
                            i += 3;
                        } else {
                            i += 2;
                        }
                    } else {
                        i += 2;
                    }
                } else {
                    i += 1;
                }
            }
            "currmove" => {
                if let Some(next) = parts.get(i + 1) {
                    info.currmove = Some(next.to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "currmovenumber" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u32>() {
                        info.currmovenumber = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "rootmoves" | "root_moves" => {
                if let Some(next) = parts.get(i + 1) {
                    if let Ok(v) = next.parse::<u32>() {
                        info.root_moves = Some(v);
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "pv" => {
                // PV extends to end of line (or next known keyword)
                let pv_start = i + 1;
                let mut pv_end = parts.len();
                for (j, &part) in parts.iter().enumerate().skip(pv_start) {
                    // Stop at next known keyword
                    if matches!(
                        part,
                        "depth"
                            | "seldepth"
                            | "time"
                            | "nodes"
                            | "nps"
                            | "hashfull"
                            | "multipv"
                            | "score"
                            | "currmove"
                            | "currmovenumber"
                            | "pv"
                            | "rootmoves"
                            | "root_moves"
                            | "string"
                    ) {
                        pv_end = j;
                        break;
                    }
                }
                info.pv = parts[pv_start..pv_end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                i = pv_end;
            }
            "string" => {
                // Skip "info string ..." - consume rest of line
                break;
            }
            _ => {
                // Unknown keyword, skip
                i += 1;
            }
        }
    }

    Some(info)
}

/// Parse all info lines from a slice of engine output lines.
pub fn parse_info_lines(lines: &[String]) -> Vec<SearchInfo> {
    lines
        .iter()
        .filter(|l| l.starts_with("info "))
        .filter_map(|l| parse_info_line(l))
        .collect()
}

/// Parse a bestmove line from a slice of engine output lines.
///
/// Returns `(bestmove, ponder)` tuple.
pub fn parse_bestmove_line(lines: &[String]) -> (Option<String>, Option<String>) {
    for line in lines {
        if let Some(rest) = line.strip_prefix("bestmove ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            let bestmove = parts.first().map(|s| s.to_string());
            let ponder = if parts.len() >= 3 && parts[1] == "ponder" {
                parts.get(2).map(|s| s.to_string())
            } else {
                None
            };
            return (bestmove, ponder);
        }
    }
    (None, None)
}

// ---------------------------------------------------------------------------
// Utility Functions
// ---------------------------------------------------------------------------

/// Standard Chinese Chess initial position FEN string.
pub const INITIAL_FEN: &str =
    "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";

/// Resolve an engine path from an environment variable or fall back to a default
/// relative to the project root.
pub fn resolve_engine_path(env_var: &str, default: &str) -> PathBuf {
    std::env::var(env_var)
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.push(default);
            p
        })
}
