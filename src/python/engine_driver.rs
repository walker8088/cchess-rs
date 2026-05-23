//! Engine driver (UCI/UCCI engine process management) for Python bindings

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// ============================================================================
// EngineOption
// ============================================================================

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

// ============================================================================
// SearchInfo
// ============================================================================

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

// ============================================================================
// SearchResult
// ============================================================================

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

// ============================================================================
// EngineProcess
// ============================================================================

/// Synchronous engine process manager for UCI/UCCI engines.
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

    #[getter]
    fn protocol(&self) -> &str {
        &self.protocol
    }

    #[getter]
    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    #[getter]
    fn engine_author(&self) -> &str {
        &self.engine_author
    }

    #[getter]
    fn options(&self) -> Vec<PyEngineOption> {
        self.options.clone()
    }

    fn send(&mut self, cmd: &str) -> PyResult<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Engine not connected"))?;
        writeln!(stdin, "{}", cmd)
            .map_err(|e| PyValueError::new_err(format!("Failed to write: {}", e)))
    }

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
                    .any(|l: &String| prefixes.iter().any(|p: &&str| l.starts_with(*p)))
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

        self.send("isready")?;
        let more = self.read_until_any(vec!["readyok"], timeout_ms)?;
        let mut all = lines;
        all.extend(more);
        Ok(all)
    }

    fn setoption(&mut self, name: &str, value: &str) -> PyResult<()> {
        let cmd = if self.protocol == "uci" {
            format!("setoption name {} value {}", name, value)
        } else {
            format!("setoption {} {}", name, value)
        };
        self.send(&cmd)
    }

    fn position_fen(&mut self, fen: &str) -> PyResult<()> {
        self.send(&format!("position fen {}", fen))
    }

    fn position_startpos_moves(&mut self, moves: &str) -> PyResult<()> {
        self.send(&format!("position startpos moves {}", moves))
    }

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
            format!("go time {}", movetime_ms / 10)
        };
        self.send(&go_cmd)?;

        let lines = self.read_until_any(vec!["bestmove", "nobestmove"], timeout_ms)?;
        self.drain_stderr();
        parse_search_result(&lines)
    }

    fn search_depth(&mut self, fen: &str, depth: u32, timeout_ms: u64) -> PyResult<PySearchResult> {
        self.position_fen(fen)?;
        self.send(&format!("go depth {}", depth))?;

        let lines = self.read_until_any(vec!["bestmove", "nobestmove"], timeout_ms)?;
        self.drain_stderr();
        parse_search_result(&lines)
    }

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
// Parsing Helper Functions
// ============================================================================

/// Parse an `option` line into an EngineOption.
pub fn parse_engine_option(rest: &str) -> PyEngineOption {
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
pub fn parse_info_line_to_py(line: &str) -> Option<PySearchInfo> {
    crate::engine_driver::parse_info_line(line).map(PySearchInfo::from)
}

/// Parse all info lines from engine output into structured PySearchInfo.
pub fn parse_info_lines_to_py(lines: &[String]) -> Vec<PySearchInfo> {
    lines
        .iter()
        .filter(|l| l.starts_with("info "))
        .filter_map(|l| parse_info_line_to_py(l))
        .collect()
}

/// Parse bestmove line from engine output.
pub fn parse_bestmove_line_from_py(lines: &[String]) -> (Option<String>, Option<String>) {
    crate::engine_driver::parse_bestmove_line(lines)
}

/// Parse all search output into a PySearchResult.
pub fn parse_search_result(lines: &[String]) -> PyResult<PySearchResult> {
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
pub fn resolve_engine_path(env_var: &str, default: &str) -> String {
    std::env::var(env_var).ok().unwrap_or_else(|| {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        format!("{}{}{}", manifest_dir, std::path::MAIN_SEPARATOR, default)
    })
}

/// Parse a single info line into a SearchInfo.
#[pyfunction]
pub fn parse_info_line(line: &str) -> Option<PySearchInfo> {
    parse_info_line_to_py(line)
}

/// Parse all info lines from a list of engine output lines.
#[pyfunction]
pub fn parse_info_lines(lines: Vec<String>) -> Vec<PySearchInfo> {
    parse_info_lines_to_py(&lines)
}

/// Parse bestmove from engine output lines.
#[pyfunction]
pub fn parse_bestmove_line(lines: Vec<String>) -> (Option<String>, Option<String>) {
    parse_bestmove_line_from_py(&lines)
}

/// Standard Chinese Chess initial position FEN string.
#[pyfunction]
pub fn initial_fen() -> String {
    crate::engine_driver::INITIAL_FEN.to_string()
}
