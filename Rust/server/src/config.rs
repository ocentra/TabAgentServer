//! Configuration system for TabAgent server.
//!
//! Supports:
//! - CLI arguments (highest priority)
//! - Environment variables  
//! - TOML config file
//! - Defaults (lowest priority)
//!
//! # RAG Compliance
//! - Uses enums for mode selection (type-safe)
//! - Proper error handling with Result
//! - No unwrap() in production code

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Context, Result};

/// Command-line arguments for TabAgent server.
#[derive(Parser, Debug, Clone)]
#[command(name = "tabagent-server")]
#[command(about = "TabAgent Server - Rust-native server with HTTP, WebRTC, and native messaging")]
#[command(version)]
pub struct CliArgs {
    /// Server mode: Native messaging, HTTP API, or Both simultaneously
    #[arg(long, short = 'm', default_value = "both", env = "TABAGENT_MODE")]
    pub mode: ServerMode,

    /// HTTP port for API server
    #[arg(long, short = 'p', default_value = "8001", env = "TABAGENT_PORT")]
    pub port: u16,

    /// Configuration file path
    #[arg(long, short = 'c', default_value = "server.toml", env = "TABAGENT_CONFIG")]
    pub config: PathBuf,

    /// Database path
    #[arg(long, default_value = "./tabagent_db", env = "TABAGENT_DB_PATH")]
    pub db_path: PathBuf,

    /// Model cache directory
    #[arg(long, default_value = "./models", env = "TABAGENT_MODEL_CACHE")]
    pub model_cache_path: PathBuf,

    /// Log level
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    pub log_level: String,

    /// Enable WebRTC
    #[arg(long, default_value = "true", env = "TABAGENT_WEBRTC_ENABLED")]
    pub webrtc_enabled: bool,

    /// WebRTC signaling port
    #[arg(long, default_value = "8002", env = "TABAGENT_WEBRTC_PORT")]
    pub webrtc_port: u16,
}

/// Server operation mode (RAG: Use enums, not strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerMode {
    /// Native messaging only (stdin/stdout for Chrome extension)
    Native,
    /// HTTP API only (for standalone server)
    Http,
    /// Both native messaging and HTTP simultaneously
    Both,
}

/// Full server configuration (merged from all sources).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server mode
    pub mode: ServerMode,
    
    /// HTTP API settings
    pub http: HttpConfig,
    
    /// WebRTC settings
    pub webrtc: WebRtcConfig,
    
    /// Database settings
    pub database: DatabaseConfig,
    
    /// Model cache settings
    pub model_cache: ModelCacheConfig,
    
    /// Python inference settings
    pub python: PythonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub port: u16,
    pub host: String,
    pub cors_origins: Vec<String>,
    pub max_body_size: usize,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    pub enabled: bool,
    pub signaling_port: u16,
    pub ice_servers: Vec<IceServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub cache_size_mb: usize,
    pub flush_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCacheConfig {
    pub path: PathBuf,
    pub max_size_gb: usize,
    pub cleanup_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    pub enabled: bool,
    pub max_workers: usize,
    pub timeout_secs: u64,
}

impl ServerConfig {
    /// Load configuration from CLI args and optional config file.
    ///
    /// Priority: CLI args > Environment > Config file > Defaults
    ///
    /// # RAG Compliance
    /// - Proper error handling with context
    /// - No unwrap() calls
    pub fn load(args: &CliArgs) -> Result<Self> {
        // Start with defaults
        let mut config = Self::default();

        // Load from config file if it exists
        if args.config.exists() {
            let file_config = Self::from_file(&args.config)
                .with_context(|| format!("Failed to load config from {:?}", args.config))?;
            config = config.merge(file_config);
        }

        // Override with CLI args (highest priority)
        config.mode = args.mode;
        config.http.port = args.port;
        config.database.path = args.db_path.clone();
        config.model_cache.path = args.model_cache_path.clone();
        config.webrtc.enabled = args.webrtc_enabled;
        config.webrtc.signaling_port = args.webrtc_port;

        Ok(config)
    }

    /// Load configuration from a TOML file.
    fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config: ServerConfig = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        
        Ok(config)
    }

    /// Merge this config with another, giving priority to the other.
    fn merge(self, other: Self) -> Self {
        // In a real implementation, we'd merge field by field
        // For now, just return other (it takes priority)
        other
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            mode: ServerMode::Both,
            http: HttpConfig {
                port: 8001,  // Note: 8001 for testing, 8000 is Python
                host: "127.0.0.1".to_string(),
                cors_origins: vec!["*".to_string()],
                max_body_size: 10 * 1024 * 1024, // 10MB
                timeout_secs: 60,
            },
            webrtc: WebRtcConfig {
                enabled: true,
                signaling_port: 8002,
                ice_servers: vec![
                    IceServer {
                        urls: vec!["stun:stun.l.google.com:19302".to_string()],
                        username: None,
                        credential: None,
                    },
                ],
            },
            database: DatabaseConfig {
                path: PathBuf::from("./tabagent_db"),
                cache_size_mb: 512,
                flush_interval_secs: 5,
            },
            model_cache: ModelCacheConfig {
                path: PathBuf::from("./models"),
                max_size_gb: 50,
                cleanup_threshold: 0.9,
            },
            python: PythonConfig {
                enabled: true,
                max_workers: 4,
                timeout_secs: 300,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.http.port, 8001);
        assert_eq!(config.mode, ServerMode::Both);
    }

    #[test]
    fn test_cli_args_override() {
        let args = CliArgs {
            mode: ServerMode::Http,
            port: 9000,
            config: PathBuf::from("nonexistent.toml"),
            db_path: PathBuf::from("./test_db"),
            model_cache_path: PathBuf::from("./test_models"),
            log_level: "debug".to_string(),
            webrtc_enabled: false,
            webrtc_port: 9002,
        };

        let config = ServerConfig::load(&args).unwrap();
        assert_eq!(config.mode, ServerMode::Http);
        assert_eq!(config.http.port, 9000);
        assert!(!config.webrtc.enabled);
    }
}

