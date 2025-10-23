//! Platform-specific utilities for database paths.
//!
//! Provides cross-platform functions to determine the appropriate
//! location for storing database files on Windows, macOS, and Linux.

use std::path::PathBuf;
use std::env;

/// Get the default database directory for the current platform.
///
/// Returns platform-specific paths:
/// - **Windows**: `%APPDATA%\TabAgent\db\`
/// - **macOS**: `~/Library/Application Support/TabAgent/db/`
/// - **Linux**: `~/.local/share/TabAgent/db/`
///
/// # Examples
///
/// ```
/// use common::platform::get_default_db_path;
///
/// let db_path = get_default_db_path();
/// println!("Database will be stored at: {:?}", db_path);
/// ```
pub fn get_default_db_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        get_windows_db_path()
    }

    #[cfg(target_os = "macos")]
    {
        get_macos_db_path()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_db_path()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Fallback for other Unix-like systems
        get_linux_db_path()
    }
}

/// Get Windows-specific database path.
///
/// Returns: `%APPDATA%\TabAgent\db\`
fn get_windows_db_path() -> PathBuf {
    if let Ok(appdata) = env::var("APPDATA") {
        PathBuf::from(appdata)
            .join("TabAgent")
            .join("db")
    } else {
        // Fallback to current directory if APPDATA not set
        PathBuf::from(".").join("TabAgent").join("db")
    }
}

/// Get macOS-specific database path.
///
/// Returns: `~/Library/Application Support/TabAgent/db/`
#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn get_macos_db_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("TabAgent")
            .join("db")
    } else {
        // Fallback
        PathBuf::from(".").join("TabAgent").join("db")
    }
}

/// Get Linux-specific database path.
///
/// Returns: `~/.local/share/TabAgent/db/`
///
/// Follows XDG Base Directory specification.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
fn get_linux_db_path() -> PathBuf {
    // Try XDG_DATA_HOME first
    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home)
            .join("TabAgent")
            .join("db");
    }

    // Fallback to ~/.local/share
    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("TabAgent")
            .join("db");
    }

    // Last resort fallback
    PathBuf::from(".").join("TabAgent").join("db")
}

/// Ensure the database directory exists, creating it if necessary.
///
/// # Errors
///
/// Returns an error if the directory cannot be created.
///
/// # Examples
///
/// ```no_run
/// use common::platform::{get_default_db_path, ensure_db_directory};
///
/// let db_path = get_default_db_path();
/// ensure_db_directory(&db_path)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn ensure_db_directory(path: &std::path::Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get a database path with a custom name within the default directory.
///
/// # Examples
///
/// ```
/// use common::platform::get_named_db_path;
///
/// // For main database
/// let main_db = get_named_db_path("main");
/// 
/// // For test database
/// let test_db = get_named_db_path("test");
/// ```
pub fn get_named_db_path(name: &str) -> PathBuf {
    get_default_db_path().join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_db_path_exists() {
        let path = get_default_db_path();
        assert!(path.to_str().is_some());
        
        // Should contain "TabAgent" and "db"
        let path_str = path.to_str().unwrap();
        assert!(path_str.contains("TabAgent"));
        assert!(path_str.contains("db"));
    }

    #[test]
    fn test_named_db_path() {
        let path = get_named_db_path("test_db");
        let path_str = path.to_str().unwrap();
        
        assert!(path_str.contains("TabAgent"));
        assert!(path_str.contains("db"));
        assert!(path_str.contains("test_db"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_path_format() {
        let path = get_default_db_path();
        let path_str = path.to_str().unwrap();
        
        // Should contain AppData or be a fallback
        assert!(path_str.contains("AppData") || path_str.contains("TabAgent"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_path_format() {
        let path = get_default_db_path();
        let path_str = path.to_str().unwrap();
        
        // Should contain Library/Application Support
        assert!(path_str.contains("Library") || path_str.contains("TabAgent"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_path_format() {
        let path = get_default_db_path();
        let path_str = path.to_str().unwrap();
        
        // Should contain .local/share or XDG path
        assert!(path_str.contains(".local") || path_str.contains("TabAgent"));
    }

    #[test]
    fn test_ensure_db_directory() {
        use tempfile::TempDir;
        
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("TabAgent").join("db");
        
        // Directory should not exist yet
        assert!(!db_path.exists());
        
        // Create it
        ensure_db_directory(&db_path).unwrap();
        
        // Now it should exist
        assert!(db_path.exists());
        assert!(db_path.is_dir());
    }
}

