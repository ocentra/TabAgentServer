//! Python bindings for TabAgent model loading
//! 
//! This crate provides Python bindings for loading and running GGUF/BitNet models
//! using Rust FFI to llama.cpp.

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use gguf_loader::{Model, ModelConfig};
use tabagent_hardware::detect_cpu_architecture;

/// Python wrapper for Model
#[pyclass]
struct PyModel {
    inner: Model,
}

// SAFETY: Model contains raw pointers to llama.cpp FFI, but llama.cpp is thread-safe
// when each thread has its own context. The Model type itself is immutable after loading
// and only accessed through &self methods. PyO3 requires Send+Sync for #[pyclass].
unsafe impl Send for PyModel {}
unsafe impl Sync for PyModel {}

#[pymethods]
impl PyModel {
    /// Load a GGUF model from file
    /// 
    /// Args:
    ///     model_path: Path to the GGUF model file
    ///     library_path: Path to llama.dll or llama.so
    ///     n_gpu_layers: Number of layers to offload to GPU (0 for CPU only, -1 for all)
    ///     use_mlock: Lock model in memory
    #[staticmethod]
    fn load(
        model_path: String,
        library_path: String,
        n_gpu_layers: Option<i32>,
        use_mlock: Option<bool>,
    ) -> PyResult<Self> {
        let mut config = ModelConfig::new(&model_path);
        
        if let Some(layers) = n_gpu_layers {
            config = config.with_gpu_layers(layers);
        }
        
        if use_mlock.unwrap_or(false) {
            config = config.with_mlock();
        }

        let model = Model::load(&library_path, config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to load model: {}", e)))?;

        Ok(PyModel { inner: model })
    }

    /// Get model vocabulary size
    fn vocab_size(&self) -> PyResult<u32> {
        Ok(self.inner.vocab_size() as u32)
    }

    /// Get model context size (training)
    fn context_train_size(&self) -> PyResult<u32> {
        Ok(self.inner.context_train_size() as u32)
    }

    /// Get model embedding dimensions
    fn embedding_dim(&self) -> PyResult<u32> {
        Ok(self.inner.embedding_dim() as u32)
    }
    
    /// Get BOS (beginning of sequence) token
    fn token_bos(&self) -> PyResult<i32> {
        Ok(self.inner.token_bos())
    }
    
    /// Get EOS (end of sequence) token
    fn token_eos(&self) -> PyResult<i32> {
        Ok(self.inner.token_eos())
    }
    
    /// Get newline token
    fn token_nl(&self) -> PyResult<i32> {
        Ok(self.inner.token_nl())
    }
}

/// Detect CPU architecture and return variant name
/// 
/// Returns:
///     String: CPU architecture variant name (e.g., "bitnet-amd-zen2")
#[pyfunction]
fn get_cpu_variant() -> PyResult<String> {
    match detect_cpu_architecture() {
        Ok(arch) => Ok(arch.variant_name().to_string()),
        Err(e) => Err(PyRuntimeError::new_err(format!("CPU detection failed: {}", e)))
    }
}

/// Get the optimal binary path for the current system
/// 
/// Args:
///     base_path: Base directory containing cpu/ subdirectories
///     binary_name: Name of the binary (e.g., "llama-server" or "llama-server.exe")
/// 
/// Returns:
///     String: Full path to the optimal binary
/// 
/// Example:
///     >>> path = get_optimal_binary("BitNet/Release", "llama-server.exe")
///     >>> # Returns: "BitNet/Release/cpu/windows/bitnet-amd-zen2/llama-server.exe"
#[pyfunction]
fn get_optimal_binary(base_path: String, binary_name: Option<String>) -> PyResult<String> {
    let binary = binary_name.unwrap_or_else(|| {
        #[cfg(target_os = "windows")]
        { "llama-server.exe".to_string() }
        
        #[cfg(not(target_os = "windows"))]
        { "llama-server".to_string() }
    });
    
    match detect_cpu_architecture() {
        Ok(arch) => {
            // Construct platform path
            #[cfg(target_os = "windows")]
            let platform = "windows";
            #[cfg(target_os = "linux")]
            let platform = "linux";
            #[cfg(target_os = "macos")]
            let platform = "macos";

            let variant = arch.variant_name();
            let binary_path = format!("{}/cpu/{}/{}/{}", base_path, platform, variant, binary);
            Ok(binary_path)
        }
        Err(e) => Err(PyRuntimeError::new_err(format!("Hardware detection failed: {}", e)))
    }
}

/// Python module definition
#[pymodule]
fn tabagent_model(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyModel>()?;
    m.add_function(wrap_pyfunction!(get_cpu_variant, m)?)?;
    m.add_function(wrap_pyfunction!(get_optimal_binary, m)?)?;
    Ok(())
}
