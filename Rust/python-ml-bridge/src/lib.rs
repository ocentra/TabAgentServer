//! ML Bridge - Python ML inference via PyO3.
//!
//! This crate provides a Rust implementation of the `MlBridge` trait that
//! calls Python ML functions via PyO3. It's the bridge between Rust's
//! high-performance core and Python's rich ML ecosystem.
//!
//! # Architecture
//!
//! ```text
//! Rust (Weaver)
//!   ├─▶ MlBridge::generate_embedding(text)
//!   │     └─▶ PyMlBridge via PyO3
//!   │           └─▶ Python: ml_funcs.generate_embedding(text)
//!   │                 └─▶ sentence-transformers model
//!   │                       └─▶ Returns Vec<f32>
//!   └─▶ Back to Rust with result
//! ```
//!
//! # Python Requirements
//!
//! The Python side must provide a module `ml_funcs` with these functions:
//! - `generate_embedding(text: str) -> list[float]`
//! - `extract_entities(text: str) -> list[dict]`
//! - `summarize(messages: list[str]) -> str`
//!
//! # Examples
//!
//! ```rust,no_run
//! use python_ml_bridge::PyMlBridge;
//! use weaver::ml_bridge::MlBridge;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize Python bridge
//!     let bridge = PyMlBridge::new("path/to/python/module")?;
//!     
//!     // Generate embedding
//!     let embedding = bridge.generate_embedding("Hello world").await?;
//!     println!("Embedding dimension: {}", embedding.len());
//!     
//!     Ok(())
//! }
//! ```

use common::DbResult;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::path::Path;
use weaver::ml_bridge::{Entity, MlBridge};

// Newtype wrapper for Entity to allow FromPyObject implementation (orphan rule)
// This keeps Entity definition in weaver (single source of truth)
// and python-ml-bridge only handles Python<->Rust conversion
#[derive(Debug)]
struct PyEntity(Entity);

impl From<PyEntity> for Entity {
    fn from(py_entity: PyEntity) -> Self {
        py_entity.0
    }
}

/// Error type for ML bridge operations.
#[derive(Debug, thiserror::Error)]
pub enum MlBridgeError {
    /// Python initialization error
    #[error("Failed to initialize Python: {0}")]
    PythonInit(String),

    /// Python module import error
    #[error("Failed to import Python module: {0}")]
    ModuleImport(String),

    /// Python function call error
    #[error("Python function call failed: {0}")]
    FunctionCall(String),

    /// Type conversion error
    #[error("Type conversion error: {0}")]
    TypeConversion(String),
}

impl From<MlBridgeError> for common::DbError {
    fn from(err: MlBridgeError) -> Self {
        common::DbError::Other(err.to_string())
    }
}

/// PyO3-based implementation of the MlBridge trait.
///
/// This struct holds references to Python functions and calls them via PyO3.
pub struct PyMlBridge {
    /// Path to the Python module directory
    module_path: String,
}

impl PyMlBridge {
    /// Creates a new PyMlBridge.
    ///
    /// # Arguments
    ///
    /// * `python_module_path` - Path to the directory containing `ml_funcs.py`
    ///
    /// # Errors
    ///
    /// Returns an error if Python initialization fails or the module cannot be imported.
    pub fn new(python_module_path: impl AsRef<Path>) -> Result<Self, MlBridgeError> {
        let module_path = python_module_path
            .as_ref()
            .to_str()
            .ok_or_else(|| MlBridgeError::PythonInit("Invalid module path".to_string()))?
            .to_string();

        // Test Python initialization
        pyo3::Python::attach(|py| {
            // Add module path to sys.path
            let sys = py.import("sys").map_err(|e| {
                MlBridgeError::ModuleImport(format!("Failed to import sys: {}", e))
            })?;
            let path_attr = sys
                .getattr("path")
                .map_err(|e| MlBridgeError::ModuleImport(format!("No sys.path: {}", e)))?;
            let sys_path = path_attr
                .cast::<PyList>()
                .map_err(|e| {
                    MlBridgeError::TypeConversion(format!("sys.path not a list: {}", e))
                })?;

            sys_path.insert(0, &module_path).map_err(|e| {
                MlBridgeError::ModuleImport(format!("Failed to add to sys.path: {}", e))
            })?;

            // Try to import the module to validate it exists
            let _ = py.import("python").map_err(|e| {
                MlBridgeError::ModuleImport(format!(
                    "Failed to import python package from {}: {}",
                    module_path, e
                ))
            })?;

            Ok::<(), MlBridgeError>(())
        })?;

        log::info!("PyMlBridge initialized with module path: {}", module_path);

        Ok(Self { module_path })
    }

    /// Convenience constructor for testing with mock module.
    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            module_path: "mock".to_string(),
        }
    }

    /// Calls a Python function and converts the result.
    fn call_python_func<'py, T>(
        &self,
        py: Python<'py>,
        func_name: &str,
        args: &Bound<'py, PyTuple>,
    ) -> Result<T, MlBridgeError>
    where
        T: for<'a, 'b> FromPyObject<'a, 'b>,
    {
        let module = py.import("python").map_err(|e| {
            MlBridgeError::FunctionCall(format!("Failed to import python package: {}", e))
        })?;

        let func = module.getattr(func_name).map_err(|e| {
            MlBridgeError::FunctionCall(format!("Function {} not found: {}", func_name, e))
        })?;

        let result = func.call1(args).map_err(|e| {
            MlBridgeError::FunctionCall(format!("Function {} failed: {}", func_name, e))
        })?;

        result.extract().map_err(|_| {
            MlBridgeError::TypeConversion(format!(
                "Failed to convert result from {}",
                func_name
            ))
        })
    }
}

#[async_trait::async_trait]
impl MlBridge for PyMlBridge {
    async fn generate_embedding(&self, text: &str) -> DbResult<Vec<f32>> {
        log::debug!("Generating embedding for {} chars", text.len());

        // Call Python function
        // Note: We use tokio::task::spawn_blocking for CPU-bound Python calls
        let text = text.to_string();
        let bridge = Self {
            module_path: self.module_path.clone(),
        };

        tokio::task::spawn_blocking(move || {
            pyo3::Python::attach(|py| {
                let args = PyTuple::new(py, &[text.into_pyobject(py).map_err(|e| common::DbError::Other(format!("Python conversion error: {}", e)))?])
                    .map_err(|e| common::DbError::Other(format!("PyTuple creation error: {}", e)))?;
                bridge
                    .call_python_func::<Vec<f32>>(py, "generate_embedding", &args)
                    .map_err(|e: MlBridgeError| -> common::DbError { e.into() })
            })
        })
        .await
        .map_err(|e| common::DbError::Other(format!("Task join error: {}", e)))?
    }

    async fn extract_entities(&self, text: &str) -> DbResult<Vec<Entity>> {
        log::debug!("Extracting entities from {} chars", text.len());

        let text = text.to_string();
        let bridge = Self {
            module_path: self.module_path.clone(),
        };

        tokio::task::spawn_blocking(move || {
            pyo3::Python::attach(|py| {
                let args = PyTuple::new(py, &[text.into_pyobject(py).map_err(|e| common::DbError::Other(format!("Python conversion error: {}", e)))?])
                    .map_err(|e| common::DbError::Other(format!("PyTuple creation error: {}", e)))?;
                
                // FromPyObject is implemented for PyEntity wrapper
                let py_entities: Vec<PyEntity> = bridge
                    .call_python_func::<Vec<PyEntity>>(py, "extract_entities", &args)
                    .map_err(|e: MlBridgeError| -> common::DbError { e.into() })?;

                // Convert PyEntity wrappers to Entity
                let entities: Vec<Entity> = py_entities.into_iter().map(Into::into).collect();

                Ok::<Vec<Entity>, common::DbError>(entities)
            })
        })
        .await
        .map_err(|e| common::DbError::Other(format!("Task join error: {}", e)))?
    }

    async fn summarize(&self, messages: &[String]) -> DbResult<String> {
        log::debug!("Summarizing {} messages", messages.len());

        let messages = messages.to_vec();
        let bridge = Self {
            module_path: self.module_path.clone(),
        };

        tokio::task::spawn_blocking(move || {
            pyo3::Python::attach(|py| {
                let args = PyTuple::new(py, &[messages.into_pyobject(py).map_err(|e| common::DbError::Other(format!("Python conversion error: {}", e)))?])
                    .map_err(|e| common::DbError::Other(format!("PyTuple creation error: {}", e)))?;
                bridge
                    .call_python_func::<String>(py, "summarize", &args)
                    .map_err(|e: MlBridgeError| -> common::DbError { e.into() })
            })
        })
        .await
        .map_err(|e| common::DbError::Other(format!("Task join error: {}", e)))?
    }

    async fn health_check(&self) -> DbResult<bool> {
        pyo3::Python::attach(|py| {
            // Check if Python is working and package is importable
            match py.import("python") {
                Ok(_) => Ok(true),
                Err(e) => {
                    log::error!("ML bridge health check failed: {}", e);
                    Ok(false)
                }
            }
        })
    }
}

// Implement FromPyObject for the newtype wrapper
impl<'a, 'py> FromPyObject<'a, 'py> for PyEntity {
    type Error = PyErr;
    
    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let any = ob.as_any();
        let dict = any.cast::<PyDict>()?;

        Ok(PyEntity(Entity {
            text: dict.get_item("text")?
                .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("Missing 'text' field"))?
                .extract()?,

            label: dict.get_item("label")?
                .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("Missing 'label' field"))?
                .extract()?,

            start: dict.get_item("start")?
                .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("Missing 'start' field"))?
                .extract()?,

            end: dict.get_item("end")?
                .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("Missing 'end' field"))?
                .extract()?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pymlbridge_creation() {
        // This test will fail if Python is not available
        // In CI/CD, you'd mock this
        let result = PyMlBridge::new("/tmp/nonexistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_bridge() {
        // Use the weaver's MockMlBridge for testing
        use weaver::ml_bridge::MockMlBridge;

        let bridge = MockMlBridge;
        let embedding = bridge.generate_embedding("test").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }
}
