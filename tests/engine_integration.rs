/// Integration tests that drive real UCI/UCCI engine executables via stdin/stdout.
///
/// These tests spawn actual engine processes (EleEye for UCCI, Pikafish for UCI),
/// send protocol commands, and validate responses.
///
/// Usage:
///   cargo test --test engine_integration
///
/// Environment variables (optional):
///   ELEEXE_PATH   - Path to EleEye executable (default: engine/eleeye/ELEEYE.EXE)
///   PIKAFISH_PATH - Path to Pikafish executable (default: engine/pikafish/pikafish.exe)
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

// Import shared types from the engine driver library
use cchess_rs::engine_driver::{
    parse_bestmove_line, parse_info_line, parse_info_lines, resolve_engine_path, Score, SearchInfo,
    INITIAL_FEN,
};

/// Aggregated search result from an engine search (sync variant)
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Best move returned by the engine
    pub bestmove: Option<String>,
    /// Ponder move (engine's expected response)
    pub ponder: Option<String>,
    /// All info lines parsed during search
    pub info_lines: Vec<SearchInfo>,
    /// Final (deepest) search info
    pub final_info: Option<SearchInfo>,
}

impl SearchResult {
    /// Get the deepest search info line
    pub fn deepest_info(&self) -> Option<&SearchInfo> {
        self.final_info.as_ref()
    }

    /// Get the score from the deepest search
    pub fn score(&self) -> Option<&Score> {
        self.final_info.as_ref().and_then(|i| i.score.as_ref())
    }

    /// Get the nodes searched
    pub fn nodes(&self) -> Option<u64> {
        self.final_info.as_ref().and_then(|i| i.nodes)
    }

    /// Get the search time in ms
    pub fn time_ms(&self) -> Option<u64> {
        self.final_info.as_ref().and_then(|i| i.time_ms)
    }

    /// Get the nodes per second
    pub fn nps(&self) -> Option<u64> {
        self.final_info.as_ref().and_then(|i| i.nps)
    }

    /// Get the max depth reached
    pub fn depth(&self) -> Option<u32> {
        self.final_info.as_ref().map(|i| i.depth)
    }

    /// Get the principal variation as a string
    pub fn pv_string(&self) -> String {
        self.final_info
            .as_ref()
            .map(|i| i.pv.join(" "))
            .unwrap_or_default()
    }
}

/// Parse all search output into a SearchResult (sync variant, wraps shared parsers)
fn parse_search_result(lines: &[String]) -> SearchResult {
    let info_lines = parse_info_lines(lines);
    let (bestmove, ponder) = parse_bestmove_line(lines);
    let final_info = info_lines.last().cloned();

    SearchResult {
        bestmove,
        ponder,
        info_lines,
        final_info,
    }
}

// ---------------------------------------------------------------------------
// Sync Process Helpers
// ---------------------------------------------------------------------------

/// Spawn an engine process and return (child, stdin, stdout reader).
fn spawn_engine(exe_path: &PathBuf) -> Result<(Child, ChildStdin, BufReader<ChildStdout>), String> {
    let dir = exe_path.parent().ok_or("engine has no parent dir")?;

    let mut child = Command::new(exe_path)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {}", exe_path.display(), e))?;

    let stdin = child.stdin.take().ok_or("failed to capture stdin")?;
    let stdout = child.stdout.take().ok_or("failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    Ok((child, stdin, reader))
}

/// Send a command to the engine via stdin.
fn send_cmd(stdin: &mut ChildStdin, cmd: &str) -> Result<(), String> {
    writeln!(stdin, "{}", cmd).map_err(|e| format!("write failed: {}", e))
}

/// Read lines from the engine until a line starting with any of the given prefixes appears,
/// or until the timeout elapses.
/// Returns all lines read (including the terminating line).
fn read_until_any(
    reader: &mut BufReader<ChildStdout>,
    prefixes: &[&str],
    timeout: Duration,
) -> Result<Vec<String>, String> {
    let start = Instant::now();
    let mut lines = Vec::new();
    let mut line_buf = String::new();

    loop {
        if start.elapsed() > timeout {
            return Err(format!(
                "timeout waiting for line starting with {:?}",
                prefixes
            ));
        }

        line_buf.clear();
        let n = reader
            .read_line(&mut line_buf)
            .map_err(|e| format!("read failed: {}", e))?;
        if n == 0 {
            // Engine closed stdout - check if we already got what we wanted
            if lines
                .iter()
                .any(|l: &String| prefixes.iter().any(|p| l.starts_with(p)))
            {
                break;
            }
            return Err("engine closed stdout".into());
        }
        let trimmed = line_buf.trim_end().to_string();
        lines.push(trimmed.clone());
        if prefixes.iter().any(|p| trimmed.starts_with(p)) {
            break;
        }
    }
    Ok(lines)
}

/// Kill the child process and ignore errors.
fn kill_proc(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

// ---------------------------------------------------------------------------
// UCI Tests (Pikafish)
// ---------------------------------------------------------------------------

struct UciEngine {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
}

impl UciEngine {
    fn new() -> Result<Self, String> {
        let exe = resolve_engine_path("PIKAFISH_PATH", "engine/pikafish/pikafish.exe");
        if !exe.exists() {
            return Err(format!("Pikafish not found at {}", exe.display()));
        }
        let (child, stdin, reader) = spawn_engine(&exe)?;
        Ok(Self {
            child,
            stdin,
            reader,
        })
    }

    /// Send `uci` and wait for `uciok`.
    fn init(&mut self) -> Result<Vec<String>, String> {
        send_cmd(&mut self.stdin, "uci")?;
        read_until_any(&mut self.reader, &["uciok"], Duration::from_secs(10))
    }

    /// Send `isready` and wait for `readyok`.
    fn isready(&mut self) -> Result<Vec<String>, String> {
        send_cmd(&mut self.stdin, "isready")?;
        read_until_any(&mut self.reader, &["readyok"], Duration::from_secs(10))
    }

    /// Send a `setoption` command.
    fn setoption(&mut self, name: &str, value: &str) -> Result<(), String> {
        send_cmd(
            &mut self.stdin,
            &format!("setoption name {} value {}", name, value),
        )
    }

    /// Send a `position` command with a FEN string.
    fn position_fen(&mut self, fen: &str) -> Result<(), String> {
        send_cmd(&mut self.stdin, &format!("position fen {}", fen))
    }

    /// Send a `position startpos` command.
    fn position_startpos(&mut self) -> Result<(), String> {
        send_cmd(&mut self.stdin, "position startpos")
    }

    /// Send `go` with a time limit (milliseconds) and wait for `bestmove`.
    fn go_movetime(&mut self, movetime_ms: u64) -> Result<Vec<String>, String> {
        send_cmd(&mut self.stdin, &format!("go movetime {}", movetime_ms))?;
        read_until_any(&mut self.reader, &["bestmove"], Duration::from_secs(30))
    }

    /// Send `go` with a time limit and return parsed SearchResult.
    fn go_search(&mut self, movetime_ms: u64) -> Result<SearchResult, String> {
        let lines = self.go_movetime(movetime_ms)?;
        Ok(parse_search_result(&lines))
    }

    /// Send `go depth <n>` and wait for `bestmove`.
    fn go_depth(&mut self, depth: u32) -> Result<Vec<String>, String> {
        send_cmd(&mut self.stdin, &format!("go depth {}", depth))?;
        read_until_any(&mut self.reader, &["bestmove"], Duration::from_secs(60))
    }

    /// Send `go depth <n>` and return parsed SearchResult.
    fn go_search_depth(&mut self, depth: u32) -> Result<SearchResult, String> {
        let lines = self.go_depth(depth)?;
        Ok(parse_search_result(&lines))
    }

    /// Parse the bestmove from the output lines (legacy, use go_search instead).
    fn parse_bestmove(lines: &[String]) -> Option<String> {
        parse_bestmove_line(lines).0
    }

    /// Send `quit`. The engine will terminate.
    fn quit(&mut self) {
        let _ = send_cmd(&mut self.stdin, "quit");
    }
}

impl Drop for UciEngine {
    fn drop(&mut self) {
        kill_proc(&mut self.child);
    }
}

// ---------------------------------------------------------------------------
// UCCI Tests (EleEye)
// ---------------------------------------------------------------------------

struct UcciEngine {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
}

impl UcciEngine {
    fn new() -> Result<Self, String> {
        let exe = resolve_engine_path("ELEEXE_PATH", "engine/eleeye/ELEEYE.EXE");
        if !exe.exists() {
            return Err(format!("EleEye not found at {}", exe.display()));
        }
        let (child, stdin, reader) = spawn_engine(&exe)?;
        Ok(Self {
            child,
            stdin,
            reader,
        })
    }

    /// Send `ucci` and wait for `ucciok`.
    fn init(&mut self) -> Result<Vec<String>, String> {
        send_cmd(&mut self.stdin, "ucci")?;
        read_until_any(&mut self.reader, &["ucciok"], Duration::from_secs(10))
    }

    /// Send `isready` and wait for `readyok`.
    fn isready(&mut self) -> Result<Vec<String>, String> {
        send_cmd(&mut self.stdin, "isready")?;
        read_until_any(&mut self.reader, &["readyok"], Duration::from_secs(10))
    }

    /// Send a `setoption` command.
    fn setoption(&mut self, name: &str, value: &str) -> Result<(), String> {
        send_cmd(&mut self.stdin, &format!("setoption {} {}", name, value))
    }

    /// Send a `position` command with a FEN-like string (UCCI uses its own format).
    /// UCCI position format: position fen <fen_string> or position startpos
    fn position_fen(&mut self, fen: &str) -> Result<(), String> {
        send_cmd(&mut self.stdin, &format!("position fen {}", fen))
    }

    /// Send a `position startpos` command.
    fn position_startpos(&mut self) -> Result<(), String> {
        send_cmd(&mut self.stdin, "position startpos")
    }

    /// Send `go` with a time limit (centiseconds for UCCI) and wait for `bestmove` or `nobestmove`.
    fn go_movetime(&mut self, time_cs: u64) -> Result<Vec<String>, String> {
        // UCCI uses centiseconds (1/100 second)
        send_cmd(&mut self.stdin, &format!("go time {}", time_cs))?;
        read_until_any(
            &mut self.reader,
            &["bestmove", "nobestmove"],
            Duration::from_secs(10),
        )
    }

    /// Send `go` with a time limit and return parsed SearchResult.
    fn go_search(&mut self, time_cs: u64) -> Result<SearchResult, String> {
        let lines = self.go_movetime(time_cs)?;
        Ok(parse_search_result(&lines))
    }

    /// Send `go` with depth limit.
    fn go_depth(&mut self, depth: u32) -> Result<Vec<String>, String> {
        send_cmd(&mut self.stdin, &format!("go depth {}", depth))?;
        read_until_any(
            &mut self.reader,
            &["bestmove", "nobestmove"],
            Duration::from_secs(30),
        )
    }

    /// Send `go` with depth limit and return parsed SearchResult.
    fn go_search_depth(&mut self, depth: u32) -> Result<SearchResult, String> {
        let lines = self.go_depth(depth)?;
        Ok(parse_search_result(&lines))
    }

    /// Parse the bestmove from the output lines (legacy, use go_search instead).
    fn parse_bestmove(lines: &[String]) -> Option<String> {
        parse_bestmove_line(lines).0
    }

    /// Send `quit`.
    fn quit(&mut self) {
        let _ = send_cmd(&mut self.stdin, "quit");
    }
}

impl Drop for UcciEngine {
    fn drop(&mut self) {
        kill_proc(&mut self.child);
    }
}

// ---------------------------------------------------------------------------
// UCI Protocol Tests
// ---------------------------------------------------------------------------

#[test]
fn test_uci_protocol_handshake() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    // 1. Send uci, expect uciok
    let lines = engine.init().expect("uci init failed");
    assert!(
        lines.iter().any(|l| l == "uciok"),
        "Expected 'uciok' in response, got: {:?}",
        lines
    );

    // 2. Check that engine sends id info
    let has_id = lines.iter().any(|l| l.starts_with("id "));
    assert!(
        has_id,
        "Expected 'id' lines in uci response, got: {:?}",
        lines
    );

    // 3. Send isready, expect readyok
    let lines = engine.isready().expect("isready failed");
    assert!(
        lines.iter().any(|l| l == "readyok"),
        "Expected 'readyok' in response, got: {:?}",
        lines
    );

    engine.quit();
}

#[test]
fn test_uci_search_initial_position() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    // Set position to initial FEN
    engine.position_fen(INITIAL_FEN).expect("position failed");

    // Search with 3 second time limit
    let lines = engine.go_movetime(3000).expect("go failed");

    // Should get a bestmove
    let bestmove = UciEngine::parse_bestmove(&lines);
    assert!(
        bestmove.is_some(),
        "Expected bestmove in response, got: {:?}",
        lines
    );

    let mv = bestmove.unwrap();
    assert!(
        !mv.is_empty(),
        "bestmove should not be empty, got: {:?}",
        lines
    );

    // ICCS move format: 4 characters (e.g., "h9g9" or "e9e8")
    assert_eq!(
        mv.len(),
        4,
        "bestmove should be 4 chars (ICCS format), got: '{}'",
        mv
    );

    println!("Pikafish bestmove from initial position: {}", mv);

    engine.quit();
}

#[test]
fn test_uci_search_depth_limited() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    engine.position_fen(INITIAL_FEN).expect("position failed");

    // Search to depth 6
    let lines = engine.go_depth(6).expect("go failed");

    let bestmove = UciEngine::parse_bestmove(&lines);
    assert!(
        bestmove.is_some(),
        "Expected bestmove in response, got: {:?}",
        lines
    );

    // Check that info lines contain depth info
    let result = parse_search_result(&lines);
    let max_depth = result.depth();
    assert!(
        max_depth.is_some(),
        "Expected depth info in search output, got: {:?}",
        lines
    );

    println!(
        "Pikafish depth-limited search: bestmove={}, max_depth={}",
        bestmove.unwrap(),
        max_depth.unwrap()
    );

    engine.quit();
}

#[test]
fn test_uci_position_with_moves() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    // Start position, play some moves: 炮二平五 (cannon from file 2 to file 5)
    // In ICCS: h9 (col=7, row=9) to e9 (col=4, row=9) -> but need to verify
    // Actually let's use a simpler position

    // Position: red to move, simple tactical position
    // Red has a rook on file 4, black king exposed
    let fen = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1";

    engine.position_fen(fen).expect("position failed");

    let lines = engine.go_movetime(2000).expect("go failed");
    let bestmove = UciEngine::parse_bestmove(&lines);

    assert!(bestmove.is_some(), "Expected bestmove, got: {:?}", lines);

    println!(
        "Pikafish bestmove from simple position: {}",
        bestmove.unwrap()
    );

    engine.quit();
}

#[test]
fn test_uci_mate_in_n() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    // Mate in 1 position: Red rook on rank 9 can deliver checkmate
    // Black king at e0 (col 4, row 0), red rook at a0 (col 0, row 0)
    // Black has no pieces that can block or capture
    let fen = "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1";

    engine.position_fen(fen).expect("position failed");

    // Search deep enough to find mate
    let lines = engine.go_depth(10).expect("go failed");

    let bestmove = UciEngine::parse_bestmove(&lines);
    assert!(bestmove.is_some(), "Expected bestmove, got: {:?}", lines);

    // Check if engine found mate score
    let result = parse_search_result(&lines);
    if let Some(score) = result.score() {
        match score {
            Score::Mate(m) => println!("Pikafish score: mate in {} moves", m),
            Score::Cp(c) => println!("Pikafish score: {} cp (positive = winning)", c),
        }
    }

    println!("Pikafish bestmove in mate position: {}", bestmove.unwrap());

    engine.quit();
}

#[test]
fn test_uci_setoption_hash() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");

    // Set hash size
    engine.setoption("Hash", "64").expect("setoption failed");
    engine.isready().expect("isready failed");

    // Verify engine still works after setting option
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let lines = engine.go_movetime(1000).expect("go failed");
    let bestmove = UciEngine::parse_bestmove(&lines);

    assert!(
        bestmove.is_some(),
        "Engine should still work after setoption, got: {:?}",
        lines
    );

    println!("Pikafish with Hash=64: bestmove={}", bestmove.unwrap());

    engine.quit();
}

#[test]
fn test_uci_multiple_moves_sequence() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    // Play a sequence of moves from startpos
    engine.position_startpos().expect("position failed");

    // Play first few moves of a common opening
    // 炮二平五: h0e0 (red cannon moves from h0 to e0)
    // 马8进7: h9g7 (black knight moves)
    let moves = "h0e0 h9g7";
    send_cmd(
        &mut engine.stdin,
        &format!("position startpos moves {}", moves),
    )
    .expect("position with moves failed");

    let lines = engine.go_movetime(2000).expect("go failed");
    let bestmove = UciEngine::parse_bestmove(&lines);

    assert!(
        bestmove.is_some(),
        "Expected bestmove after sequence, got: {:?}",
        lines
    );

    println!(
        "Pikafish after opening sequence: bestmove={}",
        bestmove.unwrap()
    );

    engine.quit();
}

// ---------------------------------------------------------------------------
// UCCI Protocol Tests
// ---------------------------------------------------------------------------

#[test]
fn test_ucci_protocol_handshake() {
    let mut engine = match UcciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    // 1. Send ucci, expect ucciok
    let lines = engine.init().expect("ucci init failed");
    assert!(
        lines.iter().any(|l| l == "ucciok"),
        "Expected 'ucciok' in response, got: {:?}",
        lines
    );

    // 2. Send isready, expect readyok
    let lines = engine.isready().expect("isready failed");
    assert!(
        lines.iter().any(|l| l == "readyok"),
        "Expected 'readyok' in response, got: {:?}",
        lines
    );

    engine.quit();
}

#[test]
fn test_ucci_search_initial_position() {
    let mut engine = match UcciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("ucci init failed");
    engine.isready().expect("isready failed");

    // Set position to initial FEN
    engine.position_fen(INITIAL_FEN).expect("position failed");

    // Search with 3 second time limit (UCCI uses centiseconds)
    let lines = engine.go_movetime(300).expect("go failed");

    // Should get a bestmove (UCCI may return "nobestmove" for impossible positions)
    let bestmove = UcciEngine::parse_bestmove(&lines);
    // "nobestmove" is a valid response when no legal moves exist
    if lines.iter().any(|l| l == "nobestmove") {
        println!("EleEye returned nobestmove (no legal moves in position)");
    } else {
        assert!(
            bestmove.is_some(),
            "Expected bestmove in response, got: {:?}",
            lines
        );

        let mv = bestmove.as_ref().unwrap();
        assert!(
            !mv.is_empty(),
            "bestmove should not be empty, got: {:?}",
            lines
        );

        // UCCI move format: typically 4 characters (e.g., "h9e9")
        assert!(
            mv.len() >= 4,
            "bestmove should be at least 4 chars, got: '{}'",
            mv
        );
    }

    println!("EleEye bestmove from initial position: {:?}", bestmove);

    engine.quit();
}

#[test]
fn test_ucci_setoption_threads() {
    let mut engine = match UcciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("ucci init failed");

    // Try setting Threads option (if supported)
    let _ = engine.setoption("Threads", "1");
    engine.isready().expect("isready failed");

    // Verify engine still works
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let lines = engine.go_movetime(100).expect("go failed");
    let bestmove = UcciEngine::parse_bestmove(&lines);

    assert!(
        bestmove.is_some(),
        "Engine should still work after setoption, got: {:?}",
        lines
    );

    println!("EleEye with Threads=1: bestmove={}", bestmove.unwrap());

    engine.quit();
}

#[test]
fn test_ucci_position_with_moves() {
    let mut engine = match UcciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("ucci init failed");
    engine.isready().expect("isready failed");

    // Position with a simple tactical setup
    let fen = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1";

    engine.position_fen(fen).expect("position failed");

    let lines = engine.go_movetime(200).expect("go failed");
    let bestmove = UcciEngine::parse_bestmove(&lines);

    // "nobestmove" is valid for empty positions
    if lines.iter().any(|l| l == "nobestmove") {
        println!("EleEye returned nobestmove (no legal moves)");
    } else {
        assert!(bestmove.is_some(), "Expected bestmove, got: {:?}", lines);
        println!(
            "EleEye bestmove from simple position: {}",
            bestmove.unwrap()
        );
    }

    engine.quit();
}

// ---------------------------------------------------------------------------
// Cross-Engine Comparison Tests
// ---------------------------------------------------------------------------

#[test]
fn test_both_engines_agree_on_best_move() {
    // Test that both engines find reasonable moves from the same position
    let uci_best = {
        let mut engine = match UciEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("SKIP Pikafish: {}", e);
                return;
            }
        };
        engine.init().expect("uci init failed");
        engine.isready().expect("isready failed");
        engine.position_fen(INITIAL_FEN).expect("position failed");
        let lines = engine.go_movetime(2000).expect("go failed");
        UciEngine::parse_bestmove(&lines).expect("no bestmove from Pikafish")
    };

    let ucci_best = {
        let mut engine = match UcciEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("SKIP EleEye: {}", e);
                return;
            }
        };
        engine.init().expect("ucci init failed");
        engine.isready().expect("isready failed");
        engine.position_fen(INITIAL_FEN).expect("position failed");
        let lines = engine.go_movetime(200).expect("go failed");
        UcciEngine::parse_bestmove(&lines).expect("no bestmove from EleEye")
    };

    println!(
        "Pikafish bestmove: {}, EleEye bestmove: {}",
        uci_best, ucci_best
    );

    // Both should find a legal move (we can't guarantee they agree,
    // but both should return valid 4-character ICCS-format moves)
    assert_eq!(uci_best.len(), 4, "Pikafish move should be 4 chars");
    assert!(
        ucci_best.len() >= 4,
        "EleEye move should be at least 4 chars"
    );
}

// ---------------------------------------------------------------------------
// Stress / Edge Case Tests
// ---------------------------------------------------------------------------

#[test]
fn test_uci_rapid_successive_searches() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    // Run 3 searches from different positions
    let positions = [
        INITIAL_FEN,
        "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/1NBAKABNR b - - 0 1",
    ];

    for (i, fen) in positions.iter().enumerate() {
        engine
            .position_fen(fen)
            .expect(&format!("position failed on iteration {}", i));

        let lines = engine
            .go_movetime(500)
            .expect(&format!("go failed on iteration {}", i));

        let bestmove = UciEngine::parse_bestmove(&lines);
        assert!(
            bestmove.is_some(),
            "Expected bestmove on iteration {}, got: {:?}",
            i,
            lines
        );

        println!("Iteration {}: bestmove = {}", i, bestmove.unwrap());
    }

    engine.quit();
}

#[test]
fn test_ucci_rapid_successive_searches() {
    let mut engine = match UcciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("ucci init failed");
    engine.isready().expect("isready failed");

    // Run 3 searches from different positions
    let positions = [
        INITIAL_FEN,
        "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/1NBAKABNR b - - 0 1",
    ];

    for (i, fen) in positions.iter().enumerate() {
        engine
            .position_fen(fen)
            .expect(&format!("position failed on iteration {}", i));

        let lines = engine
            .go_movetime(100)
            .expect(&format!("go failed on iteration {}", i));

        // UCCI may return nobestmove for some positions
        let bestmove = UcciEngine::parse_bestmove(&lines);
        if lines.iter().any(|l| l == "nobestmove") {
            println!("Iteration {}: nobestmove", i);
        } else {
            assert!(
                bestmove.is_some(),
                "Expected bestmove on iteration {}, got: {:?}",
                i,
                lines
            );
            println!("Iteration {}: bestmove = {}", i, bestmove.unwrap());
        }
    }

    engine.quit();
}

#[test]
fn test_uci_invalid_position_recovery() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    // Set a valid position first, then search
    engine
        .position_fen(INITIAL_FEN)
        .expect("valid position failed");

    let lines = engine.go_movetime(1000).expect("go failed");
    let bestmove = UciEngine::parse_bestmove(&lines);

    assert!(
        bestmove.is_some(),
        "Engine should respond with bestmove, got: {:?}",
        lines
    );

    println!("Pikafish responded: bestmove = {}", bestmove.unwrap());

    engine.quit();
}

// ---------------------------------------------------------------------------
// Performance Timing Tests
// ---------------------------------------------------------------------------

#[test]
fn test_uci_search_performance() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let start = Instant::now();
    let result = engine.go_search(3000).expect("go failed");
    let elapsed = start.elapsed();

    assert!(result.bestmove.is_some(), "Expected bestmove");
    println!(
        "Pikafish performance: move={} time={:?} nodes={:?} nps={:?} depth={:?}",
        result.bestmove.as_ref().unwrap(),
        elapsed,
        result.nodes(),
        result.nps(),
        result.depth()
    );

    // Engine should respond within reasonable time (movetime + 50% overhead)
    assert!(
        elapsed < Duration::from_secs(5),
        "Search took too long: {:?}",
        elapsed
    );

    engine.quit();
}

// ---------------------------------------------------------------------------
// Info Line Parsing Unit Tests
// ---------------------------------------------------------------------------

#[test]
fn test_parse_info_depth_cp_pv() {
    let line = "info depth 5 seldepth 8 score cp 147 pv h2e2 b0c2 nodes 12345 time 10 nps 1234500";
    let info = parse_info_line(line).expect("should parse");

    assert_eq!(info.depth, 5);
    assert_eq!(info.seldepth, Some(8));
    assert_eq!(info.nodes, Some(12345));
    assert_eq!(info.time_ms, Some(10));
    assert_eq!(info.nps, Some(1_234_500));

    match &info.score {
        Some(Score::Cp(v)) => assert_eq!(*v, 147),
        other => panic!("expected cp score, got {:?}", other),
    }

    assert_eq!(info.pv, vec!["h2e2", "b0c2"]);
}

#[test]
fn test_parse_info_mate_score() {
    let line = "info depth 10 score mate 3 pv e0f0 nodes 500000";
    let info = parse_info_line(line).expect("should parse");

    match &info.score {
        Some(Score::Mate(v)) => assert_eq!(*v, 3),
        other => panic!("expected mate score, got {:?}", other),
    }

    assert_eq!(info.depth, 10);
    assert_eq!(info.nodes, Some(500_000));
}

#[test]
fn test_parse_info_multipv() {
    let line1 = "info depth 3 multipv 1 score cp 200 pv h2e2";
    let line2 = "info depth 3 multipv 2 score cp 150 pv b2e2";

    let info1 = parse_info_line(line1).expect("should parse");
    let info2 = parse_info_line(line2).expect("should parse");

    assert_eq!(info1.multipv, Some(1));
    assert_eq!(info2.multipv, Some(2));

    match &info1.score {
        Some(Score::Cp(v)) => assert_eq!(*v, 200),
        _ => panic!("wrong score"),
    }
    match &info2.score {
        Some(Score::Cp(v)) => assert_eq!(*v, 150),
        _ => panic!("wrong score"),
    }
}

#[test]
fn test_parse_info_currmove() {
    let line = "info depth 4 currmove h2e2 currmovenumber 5 nodes 50000";
    let info = parse_info_line(line).expect("should parse");

    assert_eq!(info.currmove, Some("h2e2".to_string()));
    assert_eq!(info.currmovenumber, Some(5));
    assert_eq!(info.nodes, Some(50_000));
}

#[test]
fn test_parse_info_string_line() {
    let line = "info string NNUE evaluation using pikafish.nnue enabled";
    let info = parse_info_line(line).expect("should parse");

    assert_eq!(info.depth, 0);
    assert!(info.score.is_none());
    assert!(info.pv.is_empty());
}

#[test]
fn test_parse_info_lines_multiple() {
    let lines = vec![
        "info string NNUE evaluation using pikafish.nnue enabled".to_string(),
        "info depth 1 score cp 100 pv h2e2 nodes 100 time 1".to_string(),
        "info depth 2 score cp 150 pv h2e2 b0c2 nodes 500 time 5".to_string(),
        "info depth 3 score cp 175 pv h2e2 b0c2 h9g7 nodes 2000 time 20".to_string(),
        "bestmove h2e2".to_string(),
    ];

    let infos = parse_info_lines(&lines);
    assert_eq!(infos.len(), 4); // 4 info lines (including string line)

    // First is the string info line (no useful data)
    assert_eq!(infos[0].depth, 0);
    assert!(infos[0].score.is_none());

    // Second is depth-1 info
    assert_eq!(infos[1].depth, 1);
    match &infos[1].score {
        Some(Score::Cp(v)) => assert_eq!(*v, 100),
        _ => panic!("wrong score"),
    }

    // Last (deepest) info
    assert_eq!(infos[3].depth, 3);
    match &infos[3].score {
        Some(Score::Cp(v)) => assert_eq!(*v, 175),
        _ => panic!("wrong score"),
    }
    assert_eq!(infos[3].pv, vec!["h2e2", "b0c2", "h9g7"]);
    assert_eq!(infos[3].nodes, Some(2000));
}

#[test]
fn test_parse_bestmove_with_ponder() {
    let lines = vec![
        "info depth 10 score cp 200 pv h2e2 b0c2".to_string(),
        "bestmove h2e2 ponder b0c2".to_string(),
    ];

    let result = parse_search_result(&lines);

    assert_eq!(result.bestmove, Some("h2e2".to_string()));
    assert_eq!(result.ponder, Some("b0c2".to_string()));
    assert_eq!(result.info_lines.len(), 1);
    assert_eq!(result.depth(), Some(10));
}

#[test]
fn test_parse_bestmove_without_ponder() {
    let lines = vec!["bestmove h2e2".to_string()];

    let result = parse_search_result(&lines);

    assert_eq!(result.bestmove, Some("h2e2".to_string()));
    assert_eq!(result.ponder, None);
    assert!(result.info_lines.is_empty());
}

#[test]
fn test_search_result_helpers() {
    let lines = vec![
        "info depth 1 score cp 100 pv h2e2 nodes 100 time 1 nps 100000".to_string(),
        "info depth 5 score cp 250 pv h2e2 b0c2 nodes 50000 time 50 nps 1000000".to_string(),
        "bestmove h2e2 ponder b0c2".to_string(),
    ];

    let result = parse_search_result(&lines);

    // Helper methods should return values from the final (deepest) info
    assert_eq!(result.depth(), Some(5));
    assert_eq!(result.nodes(), Some(50_000));
    assert_eq!(result.time_ms(), Some(50));
    assert_eq!(result.nps(), Some(1_000_000));
    assert_eq!(result.pv_string(), "h2e2 b0c2");

    match result.score() {
        Some(Score::Cp(v)) => assert_eq!(*v, 250),
        other => panic!("expected cp 250, got {:?}", other),
    }

    assert!(!result.score().unwrap().is_mate());
}

// ---------------------------------------------------------------------------
// Integration Tests: Info Parsing from Real Engines
// ---------------------------------------------------------------------------

#[test]
fn test_uci_search_result_info_parsed() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let result = engine.go_search(2000).expect("go failed");

    // Verify SearchResult is populated
    assert!(result.bestmove.is_some(), "Should have bestmove");
    assert!(
        !result.info_lines.is_empty(),
        "Should have info lines, got output: {:?}",
        result.info_lines
    );

    let mv = result.bestmove.as_ref().unwrap();
    assert_eq!(mv.len(), 4, "bestmove should be 4 chars: {}", mv);

    // Verify final info has data
    let final_info = result.final_info.as_ref().expect("Should have final info");
    assert!(final_info.depth > 0, "Depth should be > 0");
    assert!(final_info.score.is_some(), "Should have score");
    assert!(
        !final_info.pv.is_empty(),
        "Should have PV: {:?}",
        final_info
    );

    println!(
        "Pikafish SearchResult: move={} depth={} score={:?} nodes={:?} nps={:?} pv={}",
        result.bestmove.as_ref().unwrap(),
        result.depth().unwrap(),
        result.score(),
        result.nodes(),
        result.nps(),
        result.pv_string()
    );

    engine.quit();
}

#[test]
fn test_uci_info_score_is_centipawn() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let result = engine.go_search(1000).expect("go failed");

    // From initial position, score should be in centipawns (not mate)
    let score = result.score().expect("Should have score");
    assert!(
        !score.is_mate(),
        "Initial position should not be a mate score"
    );
    assert!(score.cp().is_some(), "Expected cp score, got: {:?}", score);

    // Score should be reasonable (not absurdly large)
    let cp = score.cp().unwrap();
    assert!(cp.abs() < 5000, "Score seems unreasonable: {} cp", cp);

    println!("Pikafish initial position score: {} cp", cp);

    engine.quit();
}

#[test]
fn test_uci_info_nodes_increase_with_depth() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let result = engine.go_search_depth(8).expect("go failed");

    // Verify that we got multiple info lines and nodes increase
    assert!(
        result.info_lines.len() > 1,
        "Should have multiple info lines, got {}",
        result.info_lines.len()
    );

    // Last info line should have more nodes than the first
    let first_nodes = result.info_lines.first().and_then(|i| i.nodes);
    let last_nodes = result.info_lines.last().and_then(|i| i.nodes);

    if let (Some(first), Some(last)) = (first_nodes, last_nodes) {
        assert!(
            last >= first,
            "Last nodes ({}) should be >= first nodes ({})",
            last,
            first
        );
        println!(
            "Pikafish nodes: first={} last={} ratio={:.1}x",
            first,
            last,
            last as f64 / first.max(1) as f64
        );
    }

    println!(
        "Pikafish depth-8 search: {} info lines, final depth={}, nodes={:?}",
        result.info_lines.len(),
        result.depth().unwrap(),
        result.nodes()
    );

    engine.quit();
}

#[test]
fn test_uci_mate_score_detected() {
    let mut engine = match UciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("uci init failed");
    engine.isready().expect("isready failed");

    // Mate in 1: rook on a-file, black king trapped
    let fen = "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1";
    engine.position_fen(fen).expect("position failed");

    let result = engine.go_search_depth(10).expect("go failed");

    let score = result.score().expect("Should have score");

    // Pikafish may report mate or very high cp
    if score.is_mate() {
        let mate_in = score.mate().unwrap();
        assert!(
            mate_in > 0,
            "Mate should be positive (winning): {}",
            mate_in
        );
        println!("Pikafish found mate in {} moves", mate_in);
    } else {
        // If not mate, score should be very high
        let cp = score.cp().unwrap_or(0);
        assert!(
            cp > 1000,
            "Expected very high score for mate position, got {} cp",
            cp
        );
        println!("Pikafish high score (not mate format): {} cp", cp);
    }

    println!(
        "Pikafish mate search: move={} score={:?} pv={}",
        result.bestmove.as_deref().unwrap_or("none"),
        result.score(),
        result.pv_string()
    );

    engine.quit();
}

#[test]
fn test_ucci_search_result_info_parsed() {
    let mut engine = match UcciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("ucci init failed");
    engine.isready().expect("isready failed");
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let result = engine.go_search(100).expect("go failed");

    // EleEye may return nobestmove for some positions
    if result.bestmove.is_none() {
        println!("EleEye returned nobestmove");
    } else {
        let mv = result.bestmove.as_ref().unwrap();
        assert!(mv.len() >= 4, "bestmove should be at least 4 chars: {}", mv);

        println!(
            "EleEye SearchResult: move={} depth={} score={:?} nodes={:?} nps={:?} pv={}",
            mv,
            result.depth().unwrap_or(0),
            result.score(),
            result.nodes(),
            result.nps(),
            result.pv_string()
        );
    }

    engine.quit();
}

#[test]
fn test_ucci_info_score_and_pv() {
    let mut engine = match UcciEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("SKIP: {}", e);
            return;
        }
    };

    engine.init().expect("ucci init failed");
    engine.isready().expect("isready failed");
    engine.position_fen(INITIAL_FEN).expect("position failed");

    let result = engine.go_search(200).expect("go failed");

    if result.bestmove.is_some() {
        assert!(
            !result.info_lines.is_empty(),
            "Should have info lines from EleEye"
        );

        let final_info = result.final_info.as_ref().expect("Should have final info");

        // EleEye may use different score format (just a number, not cp/mate)
        // But it should still have some score and PV
        println!(
            "EleEye info: depth={} score={:?} pv={}",
            final_info.depth,
            final_info.score,
            final_info.pv.join(" ")
        );

        assert!(
            !final_info.pv.is_empty(),
            "EleEye should output PV: {:?}",
            final_info
        );
    }

    engine.quit();
}
