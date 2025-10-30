//! Python Process Manager
//!
//! Manages the lifecycle of the Python ML service subprocess.
//! Ensures the Python gRPC server starts with the Rust server and terminates when Rust exits.

use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use tracing::{info, warn, error};

/// Manager for the Python ML service subprocess
pub struct PythonProcessManager {
    process: Arc<RwLock<Option<Child>>>,
    python_dir: PathBuf,
    port: u16,
}

impl PythonProcessManager {
    /// Create a new Python process manager
    pub fn new(python_dir: PathBuf, port: u16) -> Self {
        Self {
            process: Arc::new(RwLock::new(None)),
            python_dir,
            port,
        }
    }
    
    /// Start the Python ML service
    pub async fn start(&self) -> Result<()> {
        let mut process = self.process.write().await;
        
        if process.is_some() {
            info!("Python ML service is already running");
            return Ok(());
        }
        
        info!("Starting Python ML service on port {}...", self.port);
        
        // Check if Python is available
        let python_cmd = if cfg!(target_os = "windows") {
            "python"
        } else {
            "python3"
        };
        
        // Check if ml_server.py exists
        let ml_server_path = self.python_dir.join("ml_server.py");
        if !ml_server_path.exists() {
            return Err(anyhow::anyhow!(
                "Python ML service not found at: {}. Please ensure PythonML directory is set up.",
                ml_server_path.display()
            ));
        }
        
        // Start the Python gRPC server
        let child = Command::new(python_cmd)
            .arg(&ml_server_path)
            .arg("--port")
            .arg(self.port.to_string())
            .current_dir(&self.python_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start Python ML service")?;
        
        let pid = child.id();
        *process = Some(child);
        
        info!("Python ML service started with PID: {}", pid);
        
        // Give it a moment to start up
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(())
    }
    
    /// Check if the Python service is running
    pub async fn is_running(&self) -> bool {
        let process = self.process.read().await;
        process.is_some()
    }
    
    /// Stop the Python ML service
    pub async fn stop(&self) -> Result<()> {
        let mut process = self.process.write().await;
        
        if let Some(mut child) = process.take() {
            info!("Stopping Python ML service (PID: {})...", child.id());
            
            // Try to kill gracefully
            match child.kill() {
                Ok(_) => {
                    match child.wait() {
                        Ok(status) => {
                            info!("Python ML service stopped with status: {}", status);
                        }
                        Err(e) => {
                            warn!("Failed to wait for Python ML service: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to kill Python ML service: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            info!("Python ML service was not running");
        }
        
        Ok(())
    }
    
    /// Restart the Python ML service
    pub async fn restart(&self) -> Result<()> {
        info!("Restarting Python ML service...");
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }
    
    /// Get the process ID if running
    pub async fn pid(&self) -> Option<u32> {
        let process = self.process.read().await;
        process.as_ref().map(|p| p.id())
    }
}

impl Drop for PythonProcessManager {
    fn drop(&mut self) {
        // Attempt to stop the process on drop
        if let Some(mut child) = self.process.blocking_write().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_process_manager_lifecycle() {
        let manager = PythonProcessManager::new(
            PathBuf::from("../PythonML"),
            50051
        );
        
        // Initially not running
        assert!(!manager.is_running().await);
        
        // Note: Actual start/stop testing requires Python environment
        // This is just a structure test
    }
}

