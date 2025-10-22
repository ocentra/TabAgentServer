//! Python bindings for the TabAgent Embedded Database
//!
//! This module exposes the Rust database API to Python via PyO3.

use pyo3::prelude::*;

mod types;
mod db;
mod errors;
mod query_api;
mod weaver_api;

pub use types::*;
pub use db::*;
pub use errors::*;
pub use query_api::*;
pub use weaver_api::*;

/// Python module definition
#[pymodule]
fn embedded_db(_py: Python, m: &PyModule) -> PyResult<()> {
    // Core database class
    m.add_class::<EmbeddedDB>()?;

    // Query builder classes
    m.add_class::<ConvergedQueryBuilder>()?;
    m.add_class::<StructuralFilter>()?;
    m.add_class::<GraphFilter>()?;

    // Weaver classes
    m.add_class::<WeaverController>()?;

    // Module version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
