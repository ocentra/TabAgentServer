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
    // Check if we're running in a test environment
    if let Ok(test_dir) = env::var("TABAGENT_TEST_DIR") {
        return PathBuf::from(test_dir).join("db");
    }
    
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

/// Get the quarter string for a given timestamp.
///
/// Returns a string in the format "YYYY-QN" where YYYY is the year and N is the quarter number.
/// For example: "2024-Q1", "2024-Q2", etc.
///
/// # Arguments
///
/// * `timestamp_ms` - Unix timestamp in milliseconds
///
/// # Examples
///
/// ```
/// use common::platform::get_quarter_from_timestamp;
///
/// // January 15, 2024 12:00:00 UTC
/// let timestamp = 1705320000000i64;
/// let quarter = get_quarter_from_timestamp(timestamp);
/// assert_eq!(quarter, "2024-Q1");
/// ```
pub fn get_quarter_from_timestamp(timestamp_ms: i64) -> String {
    // Convert milliseconds to seconds
    let timestamp_secs = timestamp_ms / 1000;
    
    // Create a OffsetDateTime from the timestamp
    let datetime = time::OffsetDateTime::from_unix_timestamp(timestamp_secs)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
    
    let year = datetime.year();
    let month = datetime.month() as u8;
    
    // Determine quarter (1-4)
    let quarter = match month {
        1..=3 => 1,
        4..=6 => 2,
        7..=9 => 3,
        10..=12 => 4,
        _ => 1, // Fallback, though month should always be 1-12
    };
    
    format!("{}-Q{}", year, quarter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_quarter_from_timestamp() {
        // Test Q1 (January 15, 2024)
        let q1_timestamp = 1705320000000i64; // 2024-01-15 12:00:00 UTC
        assert_eq!(get_quarter_from_timestamp(q1_timestamp), "2024-Q1");
        
        // Test Q2 (April 15, 2024)
        let q2_timestamp = 1713182400000i64; // 2024-04-15 12:00:00 UTC
        assert_eq!(get_quarter_from_timestamp(q2_timestamp), "2024-Q2");
        
        // Test Q3 (July 15, 2024)
        let q3_timestamp = 1721044800000i64; // 2024-07-15 12:00:00 UTC
        assert_eq!(get_quarter_from_timestamp(q3_timestamp), "2024-Q3");
        
        // Test Q4 (October 15, 2024)
        let q4_timestamp = 1728993600000i64; // 2024-10-15 12:00:00 UTC
        assert_eq!(get_quarter_from_timestamp(q4_timestamp), "2024-Q4");
    }
}
