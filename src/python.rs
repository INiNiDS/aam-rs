        self.inner_ref()?
            .find_obj(key)
            .map(|i| i.find_obj(key).is_some())
        self.inner_ref()?
            .validate_value(type_name, value)
            .map_err(to_py)
            .and_then(|i| i.find_obj(key).and_then(|v| v.as_object()))
            .and_then(|i| i.find_obj(key).and_then(|v| v.as_list()))
            .and_then(|i| i.find_deep(key).map(|v| v.as_str().to_string()))
            .and_then(|i| i.find_key(value).map(|v| v.as_str().to_string()))
            .and_then(|i| i.find_obj(key).map(|v| v.as_str().to_string()))
        self.inner_mut()?.merge_file(path).map_err(to_py)
        self.inner_mut()?.merge_content(content).map_err(to_py)
        let rules = FormatterRules::default();
        self.inner_ref()?.format(content, &rules).map_err(to_py)
        let report = AAM::recover_simple(content);
            .map(|inner| PyAam { inner: Some(inner) })
            .map(|inner| PyAam { inner: Some(inner) })
            inner: Some(AAM::new()),
use crate::pipeline::formatter::FormattingOptions as FormatterRules;
//! PyO3 bindings — exposes `AAM` to Python as `aam_py.AAM`.

use crate::aam::AAM;
use crate::pipeline::formatter::FormattingOptions as FormatterRules;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::collections::HashMap;

// ── Error conversion ─────────────────────────────────────────────────────────

fn to_py(err: AamError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn first_error(errors: Vec<AamError>) -> AamError {
    errors.into_iter().next().unwrap_or(AamError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    })
}

// ── PyAAM class ──────────────────────────────────────────────────────────────

#[pyclass(unsendable, name = "AAM")]
pub struct PyAam {
    inner: Option<AAM>,
}

impl PyAam {
    fn inner_ref(&self) -> PyResult<&AAM> {
        self.inner
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("AAM instance is closed"))
    }

    fn inner_mut(&mut self) -> PyResult<&mut AAM> {
        self.inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("AAM instance is closed"))
    }
}

#[pymethods]
impl PyAam {
    #[new]
    fn new() -> Self {
        PyAam {
            inner: Some(AAM::new()),
        }
    }

    #[staticmethod]
    fn parse(content: &str) -> PyResult<Self> {
        AAM::parse(content)
            .map_err(first_error)
            .map(|inner| PyAam { inner: Some(inner) })
            .map_err(to_py)
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        AAM::load(path)
            .map_err(first_error)
            .map(|inner| PyAam { inner: Some(inner) })
            .map_err(to_py)
    }

    #[staticmethod]
    fn recover_simple(content: &str) -> (Self, usize) {
        let report = AAM::recover_simple(content);
        (
            PyAam {
                inner: Some(report.recovered),
            },
            report.dropped_lines.len(),
        )
    }

    fn format(&self, content: &str) -> PyResult<String> {
        let rules = FormatterRules::default();
        self.inner_ref()?.format(content, &rules).map_err(to_py)
    }

    fn merge(&mut self, content: &str) -> PyResult<()> {
        self.inner_mut()?.merge_content(content).map_err(to_py)
    }

    fn merge_content(&mut self, content: &str) -> PyResult<()> {
        self.merge(content)
    }

    fn merge_file(&mut self, path: &str) -> PyResult<()> {
        self.inner_mut()?.merge_file(path).map_err(to_py)
    }

    fn find_obj(&self, key: &str) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|i| i.find_obj(key).map(|v| v.as_str().to_string()))
    }

    fn find_key(&self, value: &str) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|i| i.find_key(value).map(|v| v.as_str().to_string()))
    }

    fn find_deep(&self, key: &str) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|i| i.find_deep(key).map(|v| v.as_str().to_string()))
    }

    fn find_list(&self, key: &str) -> Option<Vec<String>> {
        self.inner_ref()
            .ok()
            .and_then(|i| i.find_obj(key).and_then(|v| v.as_list()))
    }

    fn find_object(&self, key: &str) -> Option<HashMap<String, String>> {
        self.inner_ref()
            .ok()
            .and_then(|i| i.find_obj(key).and_then(|v| v.as_object()))
    }

    fn keys(&self) -> Vec<String> {
        match self.inner_ref() {
            Ok(inner) => inner.keys().iter().map(|s| s.to_string()).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn to_dict(&self) -> HashMap<String, String> {
        self.inner_ref()
            .map_or_else(|_| HashMap::new(), |i| i.to_map())
    }

    fn validate_value(&self, type_name: &str, value: &str) -> PyResult<()> {
        self.inner_ref()?
            .validate_value(type_name, value)
            .map_err(to_py)
    }

    fn close(&mut self) {
        self.inner = None;
    }

    fn is_closed(&self) -> bool {
        self.inner.is_none()
    }

    fn __repr__(&self) -> String {
        match self.inner_ref() {
            Ok(inner) => format!("AAM({} keys)", inner.keys().len()),
            Err(_) => "AAM(closed)".to_string(),
        }
    }

    fn __len__(&self) -> usize {
        self.inner_ref().map_or(0, |i| i.keys().len())
    }

    fn __contains__(&self, key: &str) -> bool {
        self.inner_ref()
            .map(|i| i.find_obj(key).is_some())
            .unwrap_or(false)
    }

    fn __getitem__(&self, key: &str) -> PyResult<String> {
        self.inner_ref()?
            .find_obj(key)
            .map(|v| v.as_str().to_string())
            .ok_or_else(|| PyRuntimeError::new_err(format!("Key not found: '{key}'")))
    }
}

// ── Deprecated PyAAML class ──────────────────────────────────────────────────

#[pyclass(unsendable, name = "AAML")]
pub struct PyAaml {
    inner: PyAam,
}

#[pymethods]
impl PyAaml {
    #[new]
    #[pyo3(signature = ())]
    fn new() -> Self {
        PyAaml {
            inner: PyAam::new(),
        }
    }

    #[staticmethod]
    fn parse(content: &str) -> PyResult<Self> {
        PyAam::parse(content).map(|inner| PyAaml { inner })
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        PyAam::load(path).map(|inner| PyAaml { inner })
    }

    fn find_obj(&self, key: &str) -> Option<String> {
        self.inner.find_obj(key)
    }

    // Pass-through other commonly used methods...
    fn merge(&mut self, content: &str) -> PyResult<()> {
        self.inner.merge(content)
    }
}

pub fn register(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    m.add_class::<PyAam>()?;
    m.add_class::<PyAaml>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
