//! PyFenCache - FEN-to-action cache for storing engine recommendations

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde_json::Map as JsonMap;
use std::collections::HashMap;

use super::utils::fen_mirror;

/// Helper: convert Python value to JSON
fn python_value_to_json(py: Python<'_>, value: &PyAny) -> serde_json::Value {
    if value.is_none() {
        serde_json::Value::Null
    } else if let Ok(b) = value.extract::<bool>() {
        serde_json::Value::Bool(b)
    } else if let Ok(i) = value.extract::<i64>() {
        serde_json::Value::Number(i.into())
    } else if let Ok(f) = value.extract::<f64>() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null)
    } else if let Ok(s) = value.extract::<String>() {
        serde_json::Value::String(s)
    } else if let Ok(list) = value.downcast::<PyList>() {
        let arr: Vec<serde_json::Value> = list
            .iter()
            .map(|item| python_value_to_json(py, item))
            .collect();
        serde_json::Value::Array(arr)
    } else {
        serde_json::Value::String(value.to_string())
    }
}

/// Helper: convert JSON value to Python
fn json_value_to_python<'py>(py: Python<'py>, value: serde_json::Value) -> PyResult<PyObject> {
    Ok(match value {
        serde_json::Value::Null => py.None(),
        serde_json::Value::Bool(b) => b.into_py(py),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py(py)
            } else if let Some(f) = n.as_f64() {
                f.into_py(py)
            } else {
                py.None()
            }
        }
        serde_json::Value::String(s) => s.into_py(py),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                let py_val = json_value_to_python(py, item)?;
                list.append(py_val)
                    .map_err(|_| PyValueError::new_err("Failed to append to list"))?;
            }
            list.into()
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                let py_val = json_value_to_python(py, v)?;
                let _ = dict.set_item(&k, py_val);
            }
            dict.into()
        }
    })
}

/// Simple FEN behavior cache for storing engine recommendations.
#[pyclass(name = "FenCache")]
#[derive(Clone)]
pub struct PyFenCache {
    fen_dict: HashMap<String, Vec<(String, i64, Py<PyDict>)>>,
    cache_file: String,
    need_save: bool,
}

#[pymethods]
impl PyFenCache {
    #[new]
    pub fn new() -> Self {
        PyFenCache {
            fen_dict: HashMap::new(),
            cache_file: String::new(),
            need_save: false,
        }
    }

    /// Get cached action info for a FEN.
    /// Returns (action_dict, state_string) or (None, None).
    fn get<'py>(&'py self, py: Python<'py>, fen: &str) -> (Option<Py<PyDict>>, Option<String>) {
        if let Some(entries) = self.fen_dict.get(fen) {
            let result = PyDict::new(py);
            for (move_key, _score, action) in entries {
                let _ = result.set_item(move_key, action.as_ref(py));
            }
            return (Some(result.into()), Some("".to_string()));
        }

        let f_mirror = fen_mirror(fen);
        if let Some(entries) = self.fen_dict.get(&f_mirror) {
            let result = PyDict::new(py);
            for (move_key, _score, action) in entries {
                let _ = result.set_item(move_key, action.as_ref(py));
            }
            return (Some(result.into()), Some("mirror".to_string()));
        }

        (None, None)
    }

    /// Get the best action for a given FEN from the cache.
    #[pyo3(signature = (fen, move_color=0))]
    pub fn get_best_action<'py>(
        &self,
        py: Python<'py>,
        fen: &str,
        move_color: i32,
    ) -> PyResult<Option<Py<PyDict>>> {
        let (action_opt, state) = self.get(py, fen);
        let Some(action) = action_opt else {
            return Ok(None);
        };

        let action_ref = action.as_ref(py);
        let mut scored_actions: Vec<(i64, Py<PyDict>)> = Vec::new();

        for key in action_ref.keys() {
            let key_str: String = key.extract().unwrap_or_default();
            if key_str.starts_with('_') || key_str == "fen_engine" {
                continue;
            }
            if let Ok(Some(sub)) = action_ref.get_item(&key_str) {
                if let Ok(sub_dict) = sub.downcast::<PyDict>() {
                    if let Ok(Some(score_val)) = sub_dict.get_item("score") {
                        if let Ok(score) = score_val.extract::<i64>() {
                            let owned: Py<PyDict> = Py::from(sub_dict);
                            scored_actions.push((score, owned));
                        }
                    }
                }
            }
        }

        if scored_actions.is_empty() {
            return Ok(None);
        }

        scored_actions.sort_by_key(|(score, _)| *score);

        let best = if move_color == 1 {
            scored_actions.first().unwrap().1.clone()
        } else if move_color == -1 {
            scored_actions.last().unwrap().1.clone()
        } else {
            scored_actions.first().unwrap().1.clone()
        };

        if state.as_deref() == Some("mirror") {
            let result = super::utils::action_mirror(py, best.as_ref(py))?;
            Ok(Some(result))
        } else {
            Ok(Some(best))
        }
    }

    /// Save an action for a FEN to the cache.
    pub fn save_action(&mut self, _py: Python<'_>, fen: &str, action: &PyDict) -> PyResult<()> {
        let move_key = action
            .get_item("move")?
            .map(|v| v.extract::<String>())
            .transpose()?
            .unwrap_or_default();

        let score = action
            .get_item("score")?
            .map(|v| v.extract::<i64>())
            .transpose()?
            .unwrap_or(0);

        let owned: Py<PyDict> = Py::from(action);

        self.fen_dict
            .entry(fen.to_string())
            .or_insert_with(Vec::new)
            .push((move_key, score, owned));

        self.need_save = true;
        Ok(())
    }

    /// Save the cache to a JSON file.
    #[pyo3(signature = (path=None))]
    fn save(&self, py: Python<'_>, path: Option<&str>) -> PyResult<()> {
        let file_path = path.filter(|p| !p.is_empty()).unwrap_or(&self.cache_file);

        if file_path.is_empty() {
            return Err(PyValueError::new_err(
                "No cache file path set. Call save() with a path argument first.",
            ));
        }

        let mut json_map: JsonMap<String, serde_json::Value> = JsonMap::new();

        for (fen, entries) in &self.fen_dict {
            let mut fen_entry: JsonMap<String, serde_json::Value> = JsonMap::new();
            for (move_key, _score, action_py) in entries {
                let dict = action_py.as_ref(py);
                let mut sub_entry: JsonMap<String, serde_json::Value> = JsonMap::new();
                for key in dict.keys() {
                    let k_str: String = key.extract().unwrap_or_default();
                    if let Ok(Some(val)) = dict.get_item(&k_str) {
                        sub_entry.insert(k_str, python_value_to_json(py, val));
                    }
                }
                fen_entry.insert(move_key.clone(), serde_json::Value::Object(sub_entry));
            }
            json_map.insert(fen.clone(), serde_json::Value::Object(fen_entry));
        }

        let json_str = serde_json::to_string_pretty(&json_map)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize cache: {}", e)))?;

        std::fs::write(file_path, json_str)
            .map_err(|e| PyIOError::new_err(format!("Failed to write cache file: {}", e)))?;

        Ok(())
    }

    /// Load the cache from a JSON file.
    fn load(&mut self, py: Python<'_>, path: &str) -> PyResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PyIOError::new_err(format!("Failed to read cache file: {}", e)))?;

        let json_map: JsonMap<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse cache file: {}", e)))?;

        self.fen_dict.clear();
        for (fen, fen_value) in json_map {
            if let serde_json::Value::Object(fen_obj) = fen_value {
                let mut entries = Vec::new();
                for (move_key, sub_value) in fen_obj {
                    if let serde_json::Value::Object(sub_obj) = sub_value {
                        let dict = PyDict::new(py);
                        let score = sub_obj.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
                        for (k, v) in &sub_obj {
                            let py_val = json_value_to_python(py, v.clone())?;
                            let _ = dict.set_item(k, py_val);
                        }
                        let owned: Py<PyDict> = dict.into();
                        entries.push((move_key, score, owned));
                    }
                }
                self.fen_dict.insert(fen, entries);
            }
        }

        self.cache_file = path.to_string();
        Ok(())
    }

    #[getter]
    fn cache_file(&self) -> &str {
        &self.cache_file
    }

    #[getter]
    fn need_save(&self) -> bool {
        self.need_save
    }
}
