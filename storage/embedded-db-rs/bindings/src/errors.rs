//! Error handling and conversion between Rust and Python

use common::DbError;
use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError, PyIOError};

/// Extension trait to convert Rust DbError to Python exception
pub trait IntoPyErr {
    fn into_py_err(self) -> PyErr;
}

/// Convert Rust DbError to Python exception
impl IntoPyErr for DbError {
    fn into_py_err(self) -> PyErr {
        match self {
            DbError::NotFound(msg) => PyValueError::new_err(format!("Not found: {}", msg)),
            DbError::InvalidOperation(msg) => PyValueError::new_err(format!("Invalid operation: {}", msg)),
            DbError::Serialization(msg) => PyRuntimeError::new_err(format!("Serialization error: {}", msg)),
            DbError::Sled(err) => PyIOError::new_err(format!("Storage error: {}", err)),
            DbError::Io(err) => PyIOError::new_err(format!("IO error: {}", err)),
            DbError::Other(msg) => PyRuntimeError::new_err(format!("Database error: {}", msg)),
        }
    }
}

/// Convenience implementation to convert Result<T, DbError> to PyResult<T>
impl<T> IntoPyErr for Result<T, DbError> {
    fn into_py_err(self) -> PyErr {
        match self {
            Ok(_) => panic!("Attempted to convert Ok into error"),
            Err(e) => e.into_py_err(),
        }
    }
}

/// Helper to convert DbError results to Python
pub fn to_py_result<T>(result: Result<T, DbError>) -> PyResult<T> {
    result.map_err(|e| e.into_py_err())
}

/// Convert Python errors to DbError for internal use
pub fn py_err_to_db_error(err: PyErr) -> DbError {
    DbError::Other(format!("Python error: {}", err))
}
