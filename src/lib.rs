//! # aam-rs
//!
//! A lightweight AAML (AAM Markup Language) parser and validator.
//!
//! ## Features
//! - Simple `key = value` configuration syntax with comment support (`#`)
//! - Directive system: `@import`, `@derive`, `@schema`, `@type`
//! - Schema-based type validation — fields are checked automatically during parsing
//! - Built-in types: `i32`, `f64`, `string`, `bool`, `color`,
//!   `math::vector2/3/4`, `physics::kilogram`, `time::datetime`, and more
//! - Custom type aliases via `@type`
//! - Inheritance via `@derive` with child-wins-on-conflict semantics
//!
//! ## Quick start
//! ```no_run
//! use aam_rs::aaml::AAML;
//!
//! let cfg = AAML::load("config.aam").unwrap();
//! println!("{}", cfg.find_obj("host").unwrap());
//! ```

#[cfg(feature = "legacy")]
#[allow(deprecated)]
pub mod aaml;

pub mod builder;
pub mod commands;
pub mod error;
#[cfg(feature = "legacy")]
pub mod found_value;
pub mod pipeline;
#[cfg(feature = "legacy")]
#[allow(deprecated)]
mod types;

mod types_aam;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "jni")]
pub mod jni;
pub mod aam_value;

/// Python extension-module entry point, compiled only with `--features python`.
///
/// In Python: `from aam_py import AAML`
#[cfg(feature = "python")]
#[pyo3::pymodule(name = "aam_py")]
fn aam_py(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    python::register(m)
}
