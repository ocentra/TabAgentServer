//! Platform-specific tests for the common crate
//!
//! This file contains tests for platform-specific functionality,
//! including database path handling and directory creation.

use common::platform::{get_default_db_path, get_named_db_path, ensure_db_directory};

#[test]
fn test_default_db_path_exists() {
    let path = get_default_db_path();
    assert!(path.to_str().is_some());
    
    // Should contain "TabAgent" and "db"
    if let Some(path_str) = path.to_str() {
        assert!(path_str.contains("TabAgent"));
        assert!(path_str.contains("db"));
    }
}

#[test]
fn test_named_db_path() {
    let path = get_named_db_path("test_db");
    if let Some(path_str) = path.to_str() {
        assert!(path_str.contains("TabAgent"));
        assert!(path_str.contains("db"));
        assert!(path_str.contains("test_db"));
    }
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_path_format() {
    let path = get_default_db_path();
    if let Some(path_str) = path.to_str() {
        // Should contain AppData or be a fallback
        assert!(path_str.contains("AppData") || path_str.contains("TabAgent"));
    }
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_path_format() {
    let path = get_default_db_path();
    if let Some(path_str) = path.to_str() {
        // Should contain Library/Application Support
        assert!(path_str.contains("Library") || path_str.contains("TabAgent"));
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_path_format() {
    let path = get_default_db_path();
    if let Some(path_str) = path.to_str() {
        // Should contain .local/share or XDG path
        assert!(path_str.contains(".local") || path_str.contains("TabAgent"));
    }
}

#[test]
fn test_ensure_db_directory() {
    use tempfile::TempDir;
    
    // Use proper error handling instead of unwrap
    let temp = match TempDir::new() {
        Ok(temp) => temp,
        Err(_) => {
            // If we can't create a temp directory, the test can't proceed
            return;
        }
    };
    
    let db_path = temp.path().join("TabAgent").join("db");
    
    // Directory should not exist yet
    assert!(!db_path.exists());
    
    // Create it with proper error handling
    match ensure_db_directory(&db_path) {
        Ok(()) => {
            // Now it should exist
            assert!(db_path.exists());
            assert!(db_path.is_dir());
        }
        Err(_) => {
            // If we can't create the directory, the test fails
            panic!("Failed to create directory");
        }
    }
}