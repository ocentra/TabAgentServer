//! MDBX Environment Builder - High-level API for creating MDBX environments.
//!
//! This module provides a builder pattern to eliminate boilerplate when creating
//! MDBX environments. All low-level FFI calls are encapsulated here.
//!
//! # Example
//!
//! ```no_run
//! use mdbx_base::MdbxEnvBuilder;
//!
//! let env = MdbxEnvBuilder::new("/path/to/db")
//!     .with_max_dbs(10)
//!     .with_geometry(100 * 1024 * 1024 * 1024) // 100GB
//!     .open()?;
//! ```

use std::ffi::CString;
use std::ptr;
use mdbx_sys::{
    MDBX_env, MDBX_SUCCESS,
    mdbx_env_create, mdbx_env_set_option, mdbx_env_set_geometry, mdbx_env_open,
    mdbx_env_close_ex, MDBX_opt_max_db,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MdbxEnvError {
    #[error("mdbx_env_create failed: {0}")]
    CreateFailed(i32),
    
    #[error("mdbx_env_set_option failed: {0}")]
    SetOptionFailed(i32),
    
    #[error("mdbx_env_set_geometry failed: {0}")]
    SetGeometryFailed(i32),
    
    #[error("mdbx_env_open failed: {0}")]
    OpenFailed(i32),
    
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("invalid path: {0}")]
    InvalidPath(String),
}

/// Builder for creating MDBX environments with sensible defaults.
pub struct MdbxEnvBuilder {
    path: String,
    max_dbs: u64,
    size_lower: isize,
    size_now: isize,
    size_upper: isize,
    growth_step: isize,
    shrink_threshold: isize,
    page_size: isize,
    flags: i32,
    mode: u16,
}

impl MdbxEnvBuilder {
    /// Creates a new builder with default configuration.
    ///
    /// **Defaults:**
    /// - `max_dbs`: 10
    /// - `size_upper`: 100GB
    /// - `size_lower`, `size_now`, `growth_step`, `shrink_threshold`, `page_size`: -1 (MDBX default)
    /// - `flags`: 0 (no special flags)
    /// - `mode`: 0o600 (read/write for owner)
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            max_dbs: 10,
            size_lower: -1,
            size_now: -1,
            size_upper: 100 * 1024 * 1024 * 1024, // 100GB default
            growth_step: -1,
            shrink_threshold: -1,
            page_size: -1,
            flags: 0,
            mode: 0o600u16,
        }
    }
    
    /// Sets the maximum number of named databases (DBIs).
    ///
    /// **REQUIRED** to avoid `-30791 MDBX_DBS_FULL` error when using named databases.
    pub fn with_max_dbs(mut self, max_dbs: u64) -> Self {
        self.max_dbs = max_dbs;
        self
    }
    
    /// Sets the maximum database size in bytes.
    pub fn with_size_upper(mut self, bytes: isize) -> Self {
        self.size_upper = bytes;
        self
    }
    
    /// Sets the maximum database size in gigabytes (convenience method).
    pub fn with_size_gb(mut self, gb: u64) -> Self {
        self.size_upper = (gb * 1024 * 1024 * 1024) as isize;
        self
    }
    
    /// Sets all geometry parameters at once.
    pub fn with_geometry(
        mut self,
        size_lower: isize,
        size_now: isize,
        size_upper: isize,
        growth_step: isize,
        shrink_threshold: isize,
        page_size: isize,
    ) -> Self {
        self.size_lower = size_lower;
        self.size_now = size_now;
        self.size_upper = size_upper;
        self.growth_step = growth_step;
        self.shrink_threshold = shrink_threshold;
        self.page_size = page_size;
        self
    }
    
    /// Sets environment flags (e.g., MDBX_NOTLS, MDBX_NORDAHEAD).
    pub fn with_flags(mut self, flags: i32) -> Self {
        self.flags = flags;
        self
    }
    
    /// Sets file mode (Unix permissions).
    pub fn with_mode(mut self, mode: u16) -> Self {
        self.mode = mode;
        self
    }
    
    /// Opens the MDBX environment with the configured settings.
    ///
    /// **Automatically creates the directory if it doesn't exist.**
    ///
    /// # Returns
    ///
    /// Returns `*mut MDBX_env` on success.
    ///
    /// # Safety
    ///
    /// The returned `MDBX_env` pointer must be closed with `mdbx_env_close_ex(env, false)`
    /// when no longer needed. Consider using `MdbxEnv` wrapper for automatic cleanup.
    pub fn open(self) -> Result<*mut MDBX_env, MdbxEnvError> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&self.path)?;
        
        unsafe {
            // Step 1: Create environment
            let mut env: *mut MDBX_env = ptr::null_mut();
            let rc = mdbx_env_create(&mut env as *mut _);
            if rc != MDBX_SUCCESS {
                return Err(MdbxEnvError::CreateFailed(rc));
            }
            
            // Step 2: Set max_db option (REQUIRED for named databases!)
            let rc = mdbx_env_set_option(env, MDBX_opt_max_db, self.max_dbs);
            if rc != MDBX_SUCCESS {
                mdbx_env_close_ex(env, false);
                return Err(MdbxEnvError::SetOptionFailed(rc));
            }
            
            // Step 3: Set geometry
            let rc = mdbx_env_set_geometry(
                env,
                self.size_lower,
                self.size_now,
                self.size_upper,
                self.growth_step,
                self.shrink_threshold,
                self.page_size,
            );
            if rc != MDBX_SUCCESS {
                mdbx_env_close_ex(env, false);
                return Err(MdbxEnvError::SetGeometryFailed(rc));
            }
            
            // Step 4: Open environment
            let path_c = CString::new(self.path.clone())
                .map_err(|_| MdbxEnvError::InvalidPath(self.path.clone()))?;
            
            let rc = mdbx_env_open(env, path_c.as_ptr(), self.flags, self.mode);
            if rc != MDBX_SUCCESS {
                mdbx_env_close_ex(env, false);
                return Err(MdbxEnvError::OpenFailed(rc));
            }
            
            Ok(env)
        }
    }
}

/// RAII wrapper for MDBX_env that automatically closes on drop.
pub struct MdbxEnv {
    env: *mut MDBX_env,
}

impl MdbxEnv {
    /// Creates a new MdbxEnv from a raw pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be valid and not be closed elsewhere.
    pub unsafe fn from_raw(env: *mut MDBX_env) -> Self {
        Self { env }
    }
    
    /// Returns the raw environment pointer.
    pub fn as_ptr(&self) -> *mut MDBX_env {
        self.env
    }
    
    /// Consumes the wrapper and returns the raw pointer without closing.
    ///
    /// Caller is responsible for closing the environment.
    pub fn into_raw(self) -> *mut MDBX_env {
        let env = self.env;
        std::mem::forget(self); // Prevent Drop
        env
    }
}

impl Drop for MdbxEnv {
    fn drop(&mut self) {
        unsafe {
            if !self.env.is_null() {
                mdbx_env_close_ex(self.env, false);
            }
        }
    }
}

unsafe impl Send for MdbxEnv {}
unsafe impl Sync for MdbxEnv {}

