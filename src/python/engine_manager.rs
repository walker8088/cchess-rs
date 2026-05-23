//! PyEngineManager - Facade for loading, configuring, and querying engines

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::exceptions::PyEngineError;
use super::fen_cache::PyFenCache;

/// Engine Manager facade for loading, configuring, engines and running searches.
#[pyclass(name = "EngineManager")]
pub struct PyEngineManager {
    engine: Option<Child>,
    engine_stdin: Option<ChildStdin>,
    engine_reader: Option<BufReader<std::process::ChildStdout>>,
    protocol: String,
    go_params: Vec<(String, String)>,
    cache: PyFenCache,
    last_fen: String,
}

#[pymethods]
impl PyEngineManager {
    #[new]
    #[pyo3(signature = (cache=None))]
    fn new(cache: Option<PyFenCache>) -> Self {
        PyEngineManager {
            engine: None,
            engine_stdin: None,
            engine_reader: None,
            protocol: String::new(),
            go_params: Vec::new(),
            cache: cache.unwrap_or_else(PyFenCache::new),
            last_fen: String::new(),
        }
    }

    /// Load and initialize a UCI engine.
    #[pyo3(signature = (engine_exec, options=None, go_params=None))]
    fn load_uci(
        &mut self,
        engine_exec: &str,
        options: Option<&PyDict>,
        go_params: Option<&PyDict>,
    ) -> PyResult<bool> {
        self._load(engine_exec, "uci", options, go_params)
    }

    /// Load and initialize a UCCI engine.
    #[pyo3(signature = (engine_exec, options=None, go_params=None))]
    fn load_ucci(
        &mut self,
        engine_exec: &str,
        options: Option<&PyDict>,
        go_params: Option<&PyDict>,
    ) -> PyResult<bool> {
        self._load(engine_exec, "ucci", options, go_params)
    }

    fn _load(
        &mut self,
        engine_exec: &str,
        protocol: &str,
        options: Option<&PyDict>,
        go_params: Option<&PyDict>,
    ) -> PyResult<bool> {
        let path = PathBuf::from(engine_exec);
        if !path.exists() {
            return Err(PyValueError::new_err(format!(
                "Engine not found: {}",
                engine_exec
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
            .map_err(|e| PyEngineError::new_err(format!("Failed to spawn engine: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PyEngineError::new_err("Failed to capture stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PyEngineError::new_err("Failed to capture stdout"))?;

        let reader = BufReader::new(stdout);

        self.engine = Some(child);
        self.engine_stdin = Some(stdin);
        self.engine_reader = Some(reader);
        self.protocol = protocol.to_string();

        let init_cmd = if protocol == "uci" { "uci" } else { "ucci" };
        let ok_resp = if protocol == "uci" { "uciok" } else { "ucciok" };

        self._send_cmd(init_cmd)?;

        let ready = self._wait_for(ok_resp, 10000)?;
        if !ready {
            return Err(PyEngineError::new_err("Engine failed to initialize"));
        }

        if let Some(opts) = options {
            for item in opts.items().iter() {
                let tuple = item
                    .downcast::<pyo3::types::PyTuple>()
                    .map_err(|_| PyEngineError::new_err("Option item is not a tuple"))?;
                let key_str: String = tuple
                    .get_item(0)?
                    .extract()
                    .map_err(|_| PyValueError::new_err("Option key must be a string"))?;
                let value_str: String = tuple
                    .get_item(1)?
                    .extract()
                    .map_err(|_| PyValueError::new_err("Option value must be a string"))?;
                self._setoption(&key_str, &value_str)?;
            }
        }

        self.go_params.clear();
        if let Some(gp) = go_params {
            for item in gp.items().iter() {
                let tuple = item
                    .downcast::<pyo3::types::PyTuple>()
                    .map_err(|_| PyEngineError::new_err("Go param item is not a tuple"))?;
                let key_str: String = tuple
                    .get_item(0)?
                    .extract()
                    .map_err(|_| PyValueError::new_err("Go param key must be a string"))?;
                let value_str: String = tuple
                    .get_item(1)?
                    .extract()
                    .map_err(|_| PyValueError::new_err("Go param value must be a string"))?;
                self.go_params.push((key_str, value_str));
            }
        }

        Ok(true)
    }

    /// Get best action from cache for a FEN.
    #[pyo3(signature = (fen, move_color=0))]
    fn get_best_cache(
        &self,
        py: Python<'_>,
        fen: &str,
        move_color: i32,
    ) -> PyResult<Option<PyObject>> {
        self.cache
            .get_best_action(py, fen, move_color)
            .map(|opt| opt.map(|p| p.into()))
    }

    /// Get score/action for a FEN. Tries cache first, then runs engine.
    #[pyo3(signature = (fen, move_color=0))]
    fn get_fen_score(&mut self, py: Python<'_>, fen: &str, move_color: i32) -> PyResult<PyObject> {
        if let Some(action) = self.get_best_cache(py, fen, move_color)? {
            return Ok(action);
        }
        self.run_engine(py, fen)
    }

    /// Run the engine on a position and return the best action.
    fn run_engine(&mut self, py: Python<'_>, fen: &str) -> PyResult<PyObject> {
        if self.engine.is_none() {
            return Err(PyEngineError::new_err("Engine not loaded"));
        }

        self._send_cmd(&format!("position fen {}", fen))?;

        let mut go_cmd = String::from("go");
        for (key, value) in &self.go_params {
            go_cmd.push_str(&format!(" {} {}", key, value));
        }
        self._send_cmd(&go_cmd)?;

        let mut bestmove: Option<String> = None;
        let mut score: Option<i64> = None;
        let mut mate: Option<i32> = None;
        let mut moves_list: Vec<String> = Vec::new();

        if let Some(reader) = &mut self.engine_reader {
            let mut line = String::new();
            loop {
                line.clear();
                let n = reader
                    .read_line(&mut line)
                    .map_err(|e| PyEngineError::new_err(format!("Read error: {}", e)))?;
                if n == 0 {
                    break;
                }
                let trimmed = line.trim_end().to_string();

                if trimmed.starts_with("bestmove") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        bestmove = Some(parts[1].to_string());
                    }
                    break;
                }

                if trimmed.starts_with("info") {
                    if let Some(score_info) = trimmed.split("score ").nth(1) {
                        if let Some(cp_str) = score_info.split("cp ").nth(1) {
                            if let Some(cp_val) = cp_str.split_whitespace().next() {
                                if let Ok(cp) = cp_val.parse::<i64>() {
                                    score = Some(cp);
                                }
                            }
                        }
                        if let Some(mate_str) = score_info.split("mate ").nth(1) {
                            if let Some(mate_val) = mate_str.split_whitespace().next() {
                                if let Ok(m) = mate_val.parse::<i32>() {
                                    mate = Some(m);
                                }
                            }
                        }
                    }
                    if let Some(pv_str) = trimmed.split(" pv ").nth(1) {
                        moves_list = pv_str.split_whitespace().map(|s| s.to_string()).collect();
                    }
                }
            }
        }

        let bm =
            bestmove.ok_or_else(|| PyEngineError::new_err("Engine did not return bestmove"))?;

        let action = PyDict::new(py);
        let _ = action.set_item("move", &bm);
        if let Some(s) = score {
            let _ = action.set_item("score", -s);
        }
        if let Some(m) = mate {
            let _ = action.set_item("mate", -m);
            let checkmate_score: i64 = 30000;
            let sign: i64 = if m > 0 { 1 } else { -1 };
            let _ = action.set_item("score", (checkmate_score - m.abs() as i64) * sign);
        }
        if !moves_list.is_empty() {
            let py_moves: Vec<&str> = moves_list.iter().map(|s| s.as_str()).collect();
            let _ = action.set_item("moves", py_moves);
        }
        let _ = action.set_item("fen_engine", fen);

        let _ = self.cache.save_action(py, fen, action);

        self.last_fen = fen.to_string();

        Ok(action.into())
    }

    /// Terminate the engine.
    fn quit(&mut self) {
        let _ = self._send_cmd("quit");
        if let Some(child) = &mut self.engine {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.engine = None;
        self.engine_stdin = None;
        self.engine_reader = None;
    }

    /// Send a raw command to the engine.
    fn send_cmd(&mut self, cmd: &str) -> PyResult<()> {
        self._send_cmd(cmd)
    }

    fn __del__(&mut self) {
        self.quit();
    }
}

impl PyEngineManager {
    fn _send_cmd(&mut self, cmd: &str) -> PyResult<()> {
        let stdin = self
            .engine_stdin
            .as_mut()
            .ok_or_else(|| PyEngineError::new_err("Engine not connected"))?;
        writeln!(stdin, "{}", cmd)
            .map_err(|e| PyEngineError::new_err(format!("Failed to write: {}", e)))
    }

    fn _setoption(&mut self, name: &str, value: &str) -> PyResult<()> {
        let cmd = if self.protocol == "uci" {
            format!("setoption name {} value {}", name, value)
        } else {
            format!("setoption {} {}", name, value)
        };
        self._send_cmd(&cmd)
    }

    fn _wait_for(&mut self, prefix: &str, timeout_ms: u64) -> PyResult<bool> {
        let reader = self
            .engine_reader
            .as_mut()
            .ok_or_else(|| PyEngineError::new_err("Engine not connected"))?;

        let start = Instant::now();
        let mut line = String::new();

        loop {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return Ok(false);
            }

            line.clear();
            let n = reader
                .read_line(&mut line)
                .map_err(|e| PyEngineError::new_err(format!("Read error: {}", e)))?;
            if n == 0 {
                return Ok(false);
            }
            let trimmed = line.trim_end();
            if trimmed == prefix || trimmed.starts_with(prefix) {
                return Ok(true);
            }
        }
    }
}
