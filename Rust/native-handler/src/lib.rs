use pyo3::prelude::*;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

mod state;
mod resources;
mod defaults;

use state::{
    init_cache, get_cache, register_loaded_model, unregister_loaded_model,
    get_loaded_models, get_loaded_model, is_model_loaded, update_download_progress,
    clear_download_progress, LoadedModelInfo, LoadTarget, ModelConfigInfo,
    DownloadProgress, DownloadStatus,
    set_current_model, get_current_model,
    get_system_resources_snapshot,
};
use resources::{get_system_resources, recommend_split};
use defaults::{
    get_all_models, get_models_by_type, get_default_model_for_type,
    get_downloaded_models as defaults_get_downloaded, init_catalog,
};
use tabagent_model_cache::ModelCatalogEntry;

// Import crates
use tabagent_model_cache::ProgressCallback;
use model_loader::{Model, ModelConfig};
use tabagent_hardware::{detect_cpu_architecture, CpuArchitecture};

// Import action constants from common crate (single source of truth)
use common::actions::{
    model_lifecycle::*, rust_extended::*, status::*, message_fields::*, backends
};

/// Initialize the native handler (cache + catalog)
/// 
/// MUST be called once at startup before any other operations
/// 
/// Args:
///     cache_dir: Directory path for model cache and catalog
/// 
/// Returns: JSON response string with initialization status
#[pyfunction]
fn initialize_handler(cache_dir: &str) -> PyResult<String> {
    // Initialize model cache
    if let Err(e) = init_cache(cache_dir) {
        return Ok(json!({
            STATUS: ERROR,
            MESSAGE: format!("Failed to initialize model cache: {}", e)
        }).to_string());
    }
    
    // Initialize model catalog from default_models.json
    if let Err(e) = init_catalog(cache_dir) {
        return Ok(json!({
            STATUS: ERROR,
            MESSAGE: format!("Failed to initialize model catalog: {}", e)
        }).to_string());
    }
    
    Ok(json!({
        STATUS: SUCCESS,
        MESSAGE: "Native handler initialized successfully",
        PAYLOAD: {
            "cacheDir": cache_dir
        }
    }).to_string())
}

/// Handle a GGUF/BitNet message.
/// 
/// Python has ALREADY determined this is a GGUF/BitNet model.
/// This function MUST return a response - no Optional, no fallback.
/// 
/// Returns: JSON response string (always)
#[pyfunction]
fn handle_message(py: Python, message_json: &str) -> PyResult<String> {
    // Release GIL for async operations
    py.allow_threads(|| {
        // Create tokio runtime
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to create async runtime: {}", e)
            ))?;
        
        // Run async handler
        rt.block_on(async_handle_message(message_json))
    })
}

/// Async message handler
async fn async_handle_message(message_json: &str) -> PyResult<String> {
    // Parse incoming message
    let msg: Value = serde_json::from_str(message_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid JSON: {}", e)
        ))?;
    
    // Extract action
    let action = msg.get(ACTION)
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    // Route by action
    let response = match action {
        // Model lifecycle
        LOAD_MODEL => handle_load_model(&msg).await?,
        UNLOAD_MODEL => handle_unload_model(&msg).await?,
        DELETE_MODEL => handle_delete_model(&msg).await?,
        GENERATE => handle_generate(&msg).await?,
        GET_MODEL_STATE => handle_get_model_state(&msg).await?,
        UPDATE_SETTINGS => handle_update_settings(&msg).await?,
        STOP_GENERATION => handle_stop_generation(&msg).await?,
        PULL_MODEL | DOWNLOAD_MODEL => handle_download_model(&msg).await?,
        
        // Rust-extended actions
        GET_LOADED_MODELS => handle_get_loaded_models(&msg).await?,
        GET_DOWNLOADED_MODELS => handle_get_downloaded_models(&msg).await?,
        GET_AVAILABLE_MODELS => handle_get_available_models(&msg).await?,
        GET_SYSTEM_RESOURCES => handle_get_system_resources(&msg).await?,
        RECOMMEND_SPLIT => handle_recommend_split(&msg).await?,
        ADD_MODEL_TO_LIST => handle_add_model_to_list(&msg).await?,
        GET_CURRENT_MODEL => handle_get_current_model(&msg).await?,
        SET_ACTIVE_MODEL => handle_set_active_model(&msg).await?,
        GET_MEMORY_USAGE => handle_get_memory_usage(&msg).await?,
        GET_MODELS_BY_TYPE => handle_get_models_by_type(&msg).await?,
        GET_DEFAULT_MODEL => handle_get_default_model(&msg).await?,
        
        _ => json!({
            STATUS: ERROR,
            MESSAGE: format!("Unknown action: {}", action)
        }).to_string()
    };
    
    Ok(response)
}

// ========== CORE MODEL LOADING PIPELINE ==========

/// Handle DOWNLOAD_MODEL / PULL_MODEL action
/// Downloads a model file from HuggingFace without loading it
async fn handle_download_model(msg: &Value) -> PyResult<String> {
    let repo_id = msg.get(MODEL_PATH)
        .or_else(|| msg.get(REPO_ID))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelPath or repoId is required"
        ))?;
    
    let model_file = msg.get(MODEL_FILE)
        .or_else(|| msg.get(FILE_NAME))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelFile or fileName is required"
        ))?;
    
    // Initialize cache if needed
    ensure_cache_initialized()?;
    
    // Get cache
    let cache_lock = get_cache().map_err(|e| 
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)
    )?;
    let cache_opt = cache_lock.lock().expect("Cache mutex poisoned");
    let cache = cache_opt.as_ref().ok_or_else(||
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Cache not initialized")
    )?;
    
    // Check if already downloaded
    if cache.has_file(repo_id, model_file).map_err(|e| 
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache error: {}", e))
    )? {
        return Ok(json!({
            STATUS: SUCCESS,
            MESSAGE: "Model already downloaded",
            PAYLOAD: {
                "repoId": repo_id,
                "fileName": model_file,
                "cached": true
            }
        }).to_string());
    }
    
    // Track download start
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_secs() as i64;
    update_download_progress(DownloadProgress {
        repo_id: repo_id.to_string(),
        file_path: model_file.to_string(),
        downloaded: 0,
        total: 0,
        progress: 0,
        status: DownloadStatus::Downloading,
        started_at: start_time,
    });
    
    // Create progress callback
    let repo_id_clone = repo_id.to_string();
    let model_file_clone = model_file.to_string();
    let callback: ProgressCallback = Arc::new(move |downloaded, total| {
        let progress = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0) as u8
        } else {
            0
        };
        
        update_download_progress(DownloadProgress {
            repo_id: repo_id_clone.clone(),
            file_path: model_file_clone.clone(),
            downloaded,
            total,
            progress,
            status: DownloadStatus::Downloading,
            started_at: start_time,
        });
    });
    
    // Download
    match cache.download_file(repo_id, model_file, Some(callback)).await {
        Ok(_) => {
            clear_download_progress(repo_id, model_file);
            
            // Get file size
            let file_size = cache.get_file(repo_id, model_file).await
                .ok()
                .flatten()
                .map(|data| data.len())
                .unwrap_or(0);
            
            Ok(json!({
                STATUS: SUCCESS,
                MESSAGE: "Model downloaded successfully",
                PAYLOAD: {
                    "repoId": repo_id,
                    "fileName": model_file,
                    "size": file_size,
                    "cached": true
                }
            }).to_string())
        },
        Err(e) => {
            clear_download_progress(repo_id, model_file);
            Ok(json!({
                STATUS: ERROR,
                MESSAGE: format!("Download failed: {}", e)
            }).to_string())
        }
    }
}

/// Handle LOAD_MODEL action
/// Downloads (if needed) and loads a GGUF model into memory
async fn handle_load_model(msg: &Value) -> PyResult<String> {
    let repo_id = msg.get(MODEL_PATH)
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelPath is required"
        ))?;
    
    let model_file = msg.get(MODEL_FILE)
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelFile is required"
        ))?;
    
    // Extract architecture and task for specialized handling
    let architecture = msg.get("architecture").and_then(|v| v.as_str());
    let task = msg.get("task").and_then(|v| v.as_str());
    
    // Log routing information
    if let Some(arch) = architecture {
        eprintln!("[Rust] Loading model with architecture: {}, task: {:?}", arch, task);
    }
    
    // Generate model ID
    let model_id = format!("{}/{}", repo_id, model_file);
    
    // Check if already loaded
    if is_model_loaded(&model_id) {
        let info = get_loaded_model(&model_id)
            .expect("Model registry inconsistent: model_id exists but get_loaded_model returned None");
        return Ok(json!({
            STATUS: SUCCESS,
            MESSAGE: "Model already loaded",
            PAYLOAD: {
                IS_READY: true,
                BACKEND: backends::RUST,
                MODEL_PATH: model_id,
                "vocabSize": info.config.vocab_size,
                "contextSize": info.config.context_size,
                "loadedTo": info.loaded_to
            }
        }).to_string());
    }
    
    // Initialize cache
    ensure_cache_initialized()?;
    
    // Get cache
    let cache_lock = get_cache().map_err(|e| 
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)
    )?;
    let cache_opt = cache_lock.lock().expect("Cache mutex poisoned");
    let cache = cache_opt.as_ref().ok_or_else(||
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Cache not initialized")
    )?;
    
    // Check if model is cached, download if not
    if !cache.has_file(repo_id, model_file).map_err(|e| 
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Cache error: {}", e))
    )? {
        // Download first
        let callback: ProgressCallback = Arc::new(|_downloaded, _total| {
            // Could emit progress events here
        });
        
        cache.download_file(repo_id, model_file, Some(callback)).await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Download failed: {}", e)
            ))?;
    }
    
    // Get file path from cache
    let model_path = cache.get_file_path(repo_id, model_file).await
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to get file path: {}", e)
        ))?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Model not in cache after download"
        ))?;
    
    // Detect CPU architecture
    let cpu_arch = detect_cpu_architecture()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to detect CPU: {}", e)
        ))?;
    
    // Get optimal DLL path
    let dll_path = get_optimal_dll_path(&cpu_arch)?;
    
    // Parse settings
    let settings = msg.get("settings").and_then(|v| v.as_object());
    let n_gpu_layers = settings
        .and_then(|s| s.get("n_gpu_layers"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    
    let _n_ctx = settings
        .and_then(|s| s.get("n_ctx"))
        .and_then(|v| v.as_i64())
        .unwrap_or(4096) as usize;
    
    // Create model config
    let config = ModelConfig::new(&model_path)
        .with_gpu_layers(n_gpu_layers);
    
    // Load model
    let model = Model::load(&dll_path, config)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to load model: {}", e)
        ))?;
    
    // Get model metadata
    let vocab_size = model.vocab_size() as u32;
    let context_size = model.context_train_size() as u32;
    let embedding_dim = model.embedding_dim() as u32;
    
    // Determine load target
    let loaded_to = if n_gpu_layers > 0 {
        LoadTarget::Split { gpu_layers: n_gpu_layers as u32 }
    } else {
        LoadTarget::CPU
    };
    
    // Get file size
    let file_size = std::fs::metadata(&model_path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Register in state
    let info = LoadedModelInfo {
        model_id: model_id.clone(),
        loaded_to: loaded_to.clone(),
        gpu_layers: if n_gpu_layers > 0 { n_gpu_layers as u32 } else { 0 },
        cpu_layers: 0, // TODO: Calculate from model
        vram_used: 0, // TODO: Actual VRAM tracking
        ram_used: file_size, // Approximation
        loaded_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_secs() as i64,
        config: ModelConfigInfo {
            vocab_size: Some(vocab_size),
            context_size: Some(context_size),
            embedding_dim: Some(embedding_dim),
            file_size,
            model_type: "gguf".to_string(),
        },
    };
    
    register_loaded_model(model_id.clone(), info);
    
    // TODO: Store actual model instance for inference
    // For now, we just track metadata. Inference will be added in Phase 3.
    
    Ok(json!({
        STATUS: SUCCESS,
        MESSAGE: "Model loaded successfully",
        PAYLOAD: {
            IS_READY: true,
            BACKEND: backends::RUST,
            MODEL_PATH: model_id,
            "vocabSize": vocab_size,
            "contextSize": context_size,
            "embeddingDim": embedding_dim,
            "loadedTo": loaded_to,
            "fileSize": file_size
        }
    }).to_string())
}

/// Handle UNLOAD_MODEL action
async fn handle_unload_model(msg: &Value) -> PyResult<String> {
    let model_id = msg.get(MODEL_ID)
        .or_else(|| msg.get(MODEL_PATH))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelId or modelPath is required"
        ))?;
    
    // Check if loaded
    if !is_model_loaded(model_id) {
        return Ok(json!({
            STATUS: ERROR,
            MESSAGE: format!("Model not loaded: {}", model_id)
        }).to_string());
    }
    
    // Unregister from state
    let info = unregister_loaded_model(model_id);
    
    // TODO: Actually drop the Model instance
    // For now, we just remove from tracking
    
    Ok(json!({
        STATUS: SUCCESS,
        MESSAGE: "Model unloaded successfully",
        PAYLOAD: {
            MODEL_PATH: model_id,
            "wasLoaded": info.is_some()
        }
    }).to_string())
}

/// Handle DELETE_MODEL action
async fn handle_delete_model(msg: &Value) -> PyResult<String> {
    let repo_id = msg.get(MODEL_PATH)
        .or_else(|| msg.get(REPO_ID))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelPath or repoId is required"
        ))?;
    
    // Ensure cache initialized
    ensure_cache_initialized()?;
    
    // Get cache
    let cache_lock = get_cache().map_err(|e| 
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)
    )?;
    let cache_opt = cache_lock.lock().expect("Cache mutex poisoned");
    let cache = cache_opt.as_ref().ok_or_else(||
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Cache not initialized")
    )?;
    
    // Delete from cache
    cache.delete_model(repo_id).await
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to delete model: {}", e)
        ))?;
    
    Ok(json!({
        STATUS: SUCCESS,
        MESSAGE: "Model deleted from cache",
        PAYLOAD: {
            "repoId": repo_id
        }
    }).to_string())
}

// ========== QUERY ACTIONS ==========

/// Handle GET_LOADED_MODELS action
async fn handle_get_loaded_models(_msg: &Value) -> PyResult<String> {
    let models = get_loaded_models();
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: {
            "models": models
        }
    }).to_string())
}

/// Handle GET_MODEL_STATE action
async fn handle_get_model_state(msg: &Value) -> PyResult<String> {
    let model_id = msg.get(MODEL_ID)
        .or_else(|| msg.get(MODEL_PATH))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelId or modelPath is required"
        ))?;
    
    match get_loaded_model(model_id) {
        Some(info) => Ok(json!({
            STATUS: SUCCESS,
            PAYLOAD: info
        }).to_string()),
        None => Ok(json!({
            STATUS: ERROR,
            MESSAGE: format!("Model not loaded: {}", model_id)
        }).to_string())
    }
}

/// Handle GET_DOWNLOADED_MODELS action
async fn handle_get_downloaded_models(_msg: &Value) -> PyResult<String> {
    let downloaded = defaults_get_downloaded()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: {
            MODELS: downloaded
        }
    }).to_string())
}

/// Handle GET_AVAILABLE_MODELS action
async fn handle_get_available_models(_msg: &Value) -> PyResult<String> {
    let models = get_all_models()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: {
            MODELS: models
        }
    }).to_string())
}

/// Handle GET_SYSTEM_RESOURCES action
async fn handle_get_system_resources(_msg: &Value) -> PyResult<String> {
    let resources = get_system_resources();
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: resources
    }).to_string())
}

/// Handle RECOMMEND_SPLIT action
async fn handle_recommend_split(msg: &Value) -> PyResult<String> {
    let model_size = msg.get(MODEL_SIZE)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelSize is required"
        ))?;
    
    let total_layers = msg.get(TOTAL_LAYERS)
        .and_then(|v| v.as_u64())
        .unwrap_or(32) as u32; // Default 32 layers
    
    let recommendation = recommend_split(model_size, total_layers);
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: recommendation
    }).to_string())
}

/// Handle ADD_MODEL_TO_LIST action
async fn handle_add_model_to_list(msg: &Value) -> PyResult<String> {
    let model_data = msg.get(MODEL)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "model object is required"
        ))?;
    
    // Parse into ModelCatalogEntry
    let model: ModelCatalogEntry = serde_json::from_value(model_data.clone())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid model format: {}", e)
        ))?;
    
    defaults::add_user_model(model)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    
    Ok(json!({
        STATUS: SUCCESS,
        MESSAGE: "Model added to list"
    }).to_string())
}

/// Handle GET_CURRENT_MODEL action
async fn handle_get_current_model(_msg: &Value) -> PyResult<String> {
    let current = get_current_model();
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: {
            CURRENT_MODEL: current
        }
    }).to_string())
}

/// Handle SET_ACTIVE_MODEL action
async fn handle_set_active_model(msg: &Value) -> PyResult<String> {
    let model_id = msg.get(MODEL_ID)
        .or_else(|| msg.get(MODEL_PATH))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelId or modelPath is required"
        ))?;
    
    // Verify model is loaded
    if !is_model_loaded(model_id) {
        return Ok(json!({
            STATUS: ERROR,
            MESSAGE: format!("Model '{}' is not currently loaded", model_id)
        }).to_string());
    }
    
    set_current_model(model_id.to_string());
    
    Ok(json!({
        STATUS: SUCCESS,
        MESSAGE: format!("Active model set to '{}'", model_id)
    }).to_string())
}

/// Handle GET_MEMORY_USAGE action
async fn handle_get_memory_usage(_msg: &Value) -> PyResult<String> {
    // Get fresh system resources
    let sys_resources = get_system_resources()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to get system resources: {}", e)
        ))?;
    
    // Get snapshot if cached
    let snapshot = get_system_resources_snapshot();
    
    let ram_info = json!({
        TOTAL: sys_resources.total_ram,
        USED: sys_resources.used_ram,
        AVAILABLE: sys_resources.available_ram
    });
    
    let vram_info = sys_resources.gpu.as_ref().map(|gpu| json!({
        TOTAL: gpu.total_vram,
        USED: gpu.total_vram - gpu.available_vram,
        AVAILABLE: gpu.available_vram
    }));
    
    let payload = if let Some(snap) = snapshot {
        json!({
            RAM: ram_info,
            VRAM: vram_info,
            LOADED_MODELS_COUNT: snap.loaded_models_count,
            MEMORY_USED_BY_MODELS: snap.models_memory_usage,
            CACHED: true,
            TIMESTAMP: snap.timestamp
        })
    } else {
        json!({
            RAM: ram_info,
            VRAM: vram_info,
            LOADED_MODELS_COUNT: 0,
            MEMORY_USED_BY_MODELS: 0,
            CACHED: false
        })
    };
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: payload
    }).to_string())
}

/// Handle GET_MODELS_BY_TYPE action
async fn handle_get_models_by_type(msg: &Value) -> PyResult<String> {
    let model_type = msg.get(MODEL_TYPE)
        .or_else(|| msg.get(TYPE))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelType or type is required"
        ))?;
    
    let models = get_models_by_type(model_type)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    
    Ok(json!({
        STATUS: SUCCESS,
        PAYLOAD: {
            MODEL_TYPE: model_type,
            MODELS: models
        }
    }).to_string())
}

/// Handle GET_DEFAULT_MODEL action
async fn handle_get_default_model(msg: &Value) -> PyResult<String> {
    let model_type = msg.get(MODEL_TYPE)
        .or_else(|| msg.get(TYPE))
        .and_then(|v| v.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "modelType or type is required"
        ))?;
    
    let default_model = get_default_model_for_type(model_type)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    
    if let Some(model) = default_model {
        Ok(json!({
            STATUS: SUCCESS,
            PAYLOAD: {
                MODEL_TYPE: model_type,
                DEFAULT_MODEL: model
            }
        }).to_string())
    } else {
        Ok(json!({
            STATUS: ERROR,
            MESSAGE: format!("No default model found for type '{}'", model_type)
        }).to_string())
    }
}

// ========== NOT YET IMPLEMENTED ==========

/// Handle GENERATE action (Phase 3)
async fn handle_generate(_msg: &Value) -> PyResult<String> {
    Ok(json!({
        STATUS: ERROR,
        MESSAGE: "Generation not yet implemented (Phase 3: requires model-loader inference)"
    }).to_string())
}

/// Handle UPDATE_SETTINGS action (Phase 3)
async fn handle_update_settings(_msg: &Value) -> PyResult<String> {
    Ok(json!({
        STATUS: ERROR,
        MESSAGE: "Update settings not yet implemented"
    }).to_string())
}

/// Handle STOP_GENERATION action (Phase 3)
async fn handle_stop_generation(_msg: &Value) -> PyResult<String> {
    Ok(json!({
        STATUS: ERROR,
        MESSAGE: "Stop generation not yet implemented"
    }).to_string())
}

// ========== HELPER FUNCTIONS ==========

/// Ensure model cache is initialized
fn ensure_cache_initialized() -> PyResult<()> {
    let cache_opt = get_cache()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    
    let cache = cache_opt.lock().expect("Cache mutex poisoned");
    if cache.is_none() {
        drop(cache);
        // Initialize with default path
        init_cache("./model_cache")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    }
    
    Ok(())
}

/// Get optimal DLL path based on CPU architecture
fn get_optimal_dll_path(cpu_arch: &CpuArchitecture) -> PyResult<PathBuf> {
    // Determine platform
    #[cfg(target_os = "windows")]
    let platform = "Windows";
    #[cfg(target_os = "linux")]
    let platform = "Linux";
    #[cfg(target_os = "macos")]
    let platform = "Darwin";
    
    // Map CPU architecture to binary folder
    let cpu_variant = match cpu_arch {
        CpuArchitecture::AmdZen2 => "bitnet-amd-zen2",
        CpuArchitecture::AmdZen3 => "bitnet-amd-zen3",
        CpuArchitecture::AmdZen4 => "bitnet-amd-zen4",
        CpuArchitecture::IntelAlderlake => "bitnet-intel-alderlake",
        CpuArchitecture::IntelRocketlake => "bitnet-intel-rocketlake",
        CpuArchitecture::IntelSkylake => "bitnet-intel-skylake",
        _ => "generic", // Fallback
    };
    
    // Construct path
    #[cfg(target_os = "windows")]
    let dll_name = "llama.dll";
    #[cfg(target_os = "linux")]
    let dll_name = "llama.so";
    #[cfg(target_os = "macos")]
    let dll_name = "llama.dylib";
    
    let dll_path = PathBuf::from("BitNet")
        .join("Release")
        .join("cpu")
        .join(platform)
        .join(cpu_variant)
        .join(dll_name);
    
    // Check if exists
    if !dll_path.exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
            format!("DLL not found: {}", dll_path.display())
        ));
    }
    
    Ok(dll_path)
}

// ============================================================================
// UNIFIED MODEL DETECTION API
// ============================================================================

/// Detect model type and comprehensive task information
///
/// Performs multi-layer detection:
/// 1. Format detection (GGUF, ONNX, etc.) from file path or repo name
/// 2. Task detection from model name patterns, config.json, and HF API
///
/// Returns JSON with model information including type, backend, variants, and task
///
/// # Arguments
/// * `source` - File path or HuggingFace repository ID
/// * `auth_token` - Optional HuggingFace API token for private repos
///
/// # Returns
/// JSON string with ModelInfo structure including detected task
#[pyfunction]
fn detect_model_py(source: String, auth_token: Option<String>) -> PyResult<String> {
    use tabagent_model_cache::{
        detect_from_file_path, detect_from_repo_name,
        fetch_repo_metadata, fetch_model_config,
        detect_task_unified,
    };
    
    // Layer 1: Try file path detection first
    let model_info = detect_from_file_path(&source)
        .or_else(|| detect_from_repo_name(&source));
    
    if model_info.is_none() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Could not detect model type from: {}", source)
        ));
    }
    
    let mut info = model_info.expect("Model info should exist");
    
    // Layer 2 & 3: Enhance task detection using config.json and HF API
    // Only if source looks like a repo ID (contains /)
    if source.contains('/') && !source.contains('.') {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to create runtime: {}", e)
            ))?;
        
        runtime.block_on(async {
            // Try to fetch config.json (may not exist for all models)
            let config = fetch_model_config(&source, auth_token.as_deref()).await.ok();
            
            // Try to fetch HF API metadata for pipeline_tag
            let metadata = fetch_repo_metadata(&source, auth_token.as_deref()).await.ok();
            let pipeline_tag = metadata.as_ref().and_then(|m| m.pipeline_tag.as_deref());
            
            // Run comprehensive task detection
            let detected_task = detect_task_unified(
                &source,
                config.as_ref(),
                pipeline_tag
            );
            
            // Update model info with detected task
            info.task = Some(detected_task);
            
            Ok::<(), PyErr>(())
        })?;
    }
    
    // Serialize and return
    serde_json::to_string(&info)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to serialize model info: {}", e)
        ))
}

/// Get ONNX model manifest from HuggingFace repository
///
/// Returns JSON with extension-compatible manifest including all quantizations
///
/// # Arguments
/// * `repo` - Repository ID (e.g., "microsoft/Phi-3-mini-4k-instruct-onnx")
/// * `auth_token` - Optional HuggingFace API token
/// * `server_only_size_limit` - Optional size limit in bytes (default 2.1GB)
///
/// # Returns
/// JSON string with ExtensionManifestEntry structure
#[pyfunction]
fn get_model_manifest_py(
    repo: String,
    auth_token: Option<String>,
    server_only_size_limit: Option<u64>,
) -> PyResult<String> {
    use tabagent_model_cache::{
        fetch_repo_metadata, build_manifest_from_hf,
        DEFAULT_SERVER_ONLY_SIZE, DEFAULT_BYPASS_MODELS,
    };
    
    // Use tokio runtime for async call
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    runtime.block_on(async {
        // Fetch metadata from HuggingFace
        let metadata = fetch_repo_metadata(
            &repo,
            auth_token.as_deref()
        ).await.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to fetch repo metadata: {}", e)
        ))?;
        
        // Build manifest
        let size_limit = server_only_size_limit.unwrap_or(DEFAULT_SERVER_ONLY_SIZE);
        let manifest = build_manifest_from_hf(
            &metadata,
            size_limit,
            DEFAULT_BYPASS_MODELS
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to build manifest: {}", e)
        ))?;
        
        // Serialize to JSON
        serde_json::to_string(&manifest)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to serialize manifest: {}", e)
            ))
    })
}

/// Recommend optimal model variant based on hardware capabilities
///
/// Returns the recommended quantization key based on available RAM/VRAM
///
/// # Arguments
/// * `repo` - Repository ID
/// * `available_ram_gb` - Available system RAM in GB
/// * `available_vram_gb` - Available VRAM in GB (0 if no GPU)
///
/// # Returns
/// Recommended quant key (e.g., "onnx/model_q4f16.onnx")
#[pyfunction]
fn recommend_variant_py(
    repo: String,
    _available_ram_gb: f32,
    _available_vram_gb: f32,
) -> PyResult<String> {
    use tabagent_model_cache::{fetch_repo_metadata, build_manifest_from_hf, DEFAULT_SERVER_ONLY_SIZE, DEFAULT_BYPASS_MODELS};
    
    // Use tokio runtime for async call
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    runtime.block_on(async {
        // Fetch and build manifest
        let metadata = fetch_repo_metadata(&repo, None).await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to fetch repo metadata: {}", e)
            ))?;
        
        let manifest = build_manifest_from_hf(&metadata, DEFAULT_SERVER_ONLY_SIZE, DEFAULT_BYPASS_MODELS)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to build manifest: {}", e)
            ))?;
        
        // Simple selection logic:
        // 1. Prefer q4f16 for good balance
        // 2. Fall back to fp16 if available
        // 3. Use fp32 as last resort
        let preference_order = vec!["q4f16", "q4", "fp16", "fp32"];
        
        for dtype in preference_order {
            for (quant_key, quant_info) in &manifest.quants {
                if quant_info.dtype.contains(dtype) {
                    return Ok(quant_key.clone());
                }
            }
        }
        
        // If no match, return first available quant
        manifest.quants.keys().next()
            .cloned()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No quantizations found in manifest"
            ))
    })
}

/// Python module definition
#[pymodule]
fn tabagent_native_handler(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(initialize_handler, m)?)?;
    m.add_function(wrap_pyfunction!(handle_message, m)?)?;
    
    // Unified API functions
    m.add_function(wrap_pyfunction!(detect_model_py, m)?)?;
    m.add_function(wrap_pyfunction!(get_model_manifest_py, m)?)?;
    m.add_function(wrap_pyfunction!(recommend_variant_py, m)?)?;
    
    Ok(())
}
