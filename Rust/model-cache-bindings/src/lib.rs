use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use ::tabagent_model_cache::{ModelCache, ManifestEntry, QuantStatus, ProgressCallback};
use std::sync::Arc;

/// Python wrapper for ModelCache
#[pyclass(name = "ModelCache")]
struct PyModelCache {
    cache: Arc<ModelCache>,
    runtime: Arc<tokio::runtime::Runtime>,
}

#[pymethods]
impl PyModelCache {
    #[new]
    fn new(db_path: String) -> PyResult<Self> {
        let cache = ModelCache::new(&db_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create cache: {}", e)))?;
        
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;
        
        Ok(Self {
            cache: Arc::new(cache),
            runtime: Arc::new(runtime),
        })
    }
    
    /// Scan a HuggingFace repository and update manifest
    fn scan_repo(&self, repo_id: String) -> PyResult<PyObject> {
        let cache = Arc::clone(&self.cache);
        
        let manifest = self.runtime.block_on(async move {
            cache.scan_repo(&repo_id).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to scan repo: {}", e)))?;
        
        Python::with_gil(|py| {
            manifest_to_dict(py, &manifest)
        })
    }
    
    /// Get manifest for a repository
    fn get_manifest(&self, repo_id: String) -> PyResult<Option<PyObject>> {
        let cache = Arc::clone(&self.cache);
        
        let manifest = self.runtime.block_on(async move {
            cache.get_manifest(&repo_id).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get manifest: {}", e)))?;
        
        match manifest {
            Some(m) => Python::with_gil(|py| Ok(Some(manifest_to_dict(py, &m)?))),
            None => Ok(None),
        }
    }
    
    /// Download a specific file
    fn download_file(&self, repo_id: String, file_path: String, progress_callback: Option<PyObject>) -> PyResult<()> {
        let cache = Arc::clone(&self.cache);
        
        // Convert Python callback to Rust callback
        let callback = progress_callback.map(|cb| {
            Arc::new(move |loaded: u64, total: u64| {
                Python::with_gil(|py| {
                    let _ = cb.call1(py, (loaded, total));
                });
            }) as ProgressCallback
        });
        
        self.runtime.block_on(async move {
            cache.download_file(&repo_id, &file_path, callback).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to download file: {}", e)))?;
        
        Ok(())
    }
    
    /// Download all files for a quantization variant
    fn download_quant(&self, repo_id: String, quant_key: String, progress_callback: Option<PyObject>) -> PyResult<()> {
        let cache = Arc::clone(&self.cache);
        
        let callback = progress_callback.map(|cb| {
            Arc::new(move |loaded: u64, total: u64| {
                Python::with_gil(|py| {
                    let _ = cb.call1(py, (loaded, total));
                });
            }) as ProgressCallback
        });
        
        self.runtime.block_on(async move {
            cache.download_quant(&repo_id, &quant_key, callback).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to download quant: {}", e)))?;
        
        Ok(())
    }
    
    /// Get a cached file
    fn get_file(&self, repo_id: String, file_path: String) -> PyResult<Option<Vec<u8>>> {
        let cache = self.cache.clone();
        
        self.runtime.block_on(async move {
            cache.get_file(&repo_id, &file_path).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get file: {}", e)))
    }
    
    /// Check if a file is cached
    fn has_file(&self, repo_id: String, file_path: String) -> PyResult<bool> {
        self.cache.has_file(&repo_id, &file_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to check file: {}", e)))
    }
    
    /// Delete a model and all its files
    fn delete_model(&self, repo_id: String) -> PyResult<()> {
        let cache = Arc::clone(&self.cache);
        
        self.runtime.block_on(async move {
            cache.delete_model(&repo_id).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to delete model: {}", e)))?;
        
        Ok(())
    }
    
    /// Get cache statistics
    fn get_stats(&self) -> PyResult<PyObject> {
        let cache = Arc::clone(&self.cache);
        
        let stats = self.runtime.block_on(async move {
            cache.get_stats().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get stats: {}", e)))?;
        
        Python::with_gil(|py| {
            let dict = PyDict::new_bound(py);
            dict.set_item("total_repos", stats.total_repos)?;
            dict.set_item("total_size", stats.total_size)?;
            Ok(dict.into())
        })
    }
}

/// Convert ManifestEntry to Python dict
fn manifest_to_dict(py: Python, manifest: &ManifestEntry) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);
    dict.set_item("repo_id", &manifest.repo_id)?;
    dict.set_item("task", &manifest.task)?;
    dict.set_item("created_at", manifest.created_at)?;
    dict.set_item("updated_at", manifest.updated_at)?;
    
    // Convert quants HashMap to dict
    let quants_dict = PyDict::new_bound(py);
    for (key, info) in &manifest.quants {
        let info_dict = PyDict::new_bound(py);
        
        let status_str = match info.status {
            QuantStatus::Available => "available",
            QuantStatus::Downloading => "downloading",
            QuantStatus::Downloaded => "downloaded",
            QuantStatus::Failed => "failed",
            QuantStatus::Unsupported => "unsupported",
            QuantStatus::NotFound => "not_found",
            QuantStatus::Unavailable => "unavailable",
            QuantStatus::ServerOnly => "server_only",
        };
        
        info_dict.set_item("status", status_str)?;
        info_dict.set_item("files", PyList::new_bound(py, &info.files))?;
        info_dict.set_item("total_size", info.total_size)?;
        info_dict.set_item("downloaded_size", info.downloaded_size)?;
        info_dict.set_item("last_updated", info.last_updated)?;
        
        quants_dict.set_item(key, info_dict)?;
    }
    
    dict.set_item("quants", quants_dict)?;
    
    Ok(dict.into())
}

#[pymodule]
fn tabagent_model_cache_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyModelCache>()?;
    Ok(())
}

