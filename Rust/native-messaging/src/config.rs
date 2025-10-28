//! Configuration management for native messaging host.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for the native messaging host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeMessagingConfig {
    /// Maximum message size in bytes (Chrome limit is 1MB)
    pub max_message_size: usize,
    
    /// Enable request/response logging
    pub enable_logging: bool,
    
    /// Log level for the host
    pub log_level: String,
    
    /// Enable development mode features
    pub dev_mode: bool,
    
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    
    /// Requests per minute for standard tier
    pub standard_rpm: u32,
    
    /// Requests per minute for inference tier
    pub inference_rpm: u32,
    
    /// Requests per minute for premium tier
    pub premium_rpm: u32,
}

/// Security configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable Chrome extension origin validation
    pub validate_origins: bool,
    
    /// Allowed Chrome extension IDs (empty = allow all)
    pub allowed_extension_ids: Vec<String>,
    
    /// Enable authentication for protected routes
    pub enable_auth: bool,
    
    /// Enable audit logging
    pub audit_logging: bool,
}

impl Default for NativeMessagingConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1_048_576, // 1MB Chrome limit
            enable_logging: true,
            log_level: "info".to_string(),
            dev_mode: false,
            rate_limiting: RateLimitConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            standard_rpm: 60,    // 1 request per second
            inference_rpm: 30,   // 1 request per 2 seconds
            premium_rpm: 120,    // 2 requests per second
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            validate_origins: true,
            allowed_extension_ids: Vec::new(), // Empty = allow all
            enable_auth: false, // TODO: Enable when auth is implemented
            audit_logging: true,
        }
    }
}

impl NativeMessagingConfig {
    /// Load configuration from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file (JSON or TOML)
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        
        let config = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)?
        } else {
            // Default to JSON
            serde_json::from_str(&content)?
        };
        
        Ok(config)
    }
    
    /// Save configuration to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();
        
        let content = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(self)?
        } else {
            // Default to JSON
            serde_json::to_string_pretty(self)?
        };
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Validate the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_message_size == 0 {
            return Err(anyhow::anyhow!("max_message_size must be greater than 0"));
        }
        
        if self.max_message_size > 1_048_576 {
            return Err(anyhow::anyhow!("max_message_size cannot exceed Chrome's 1MB limit"));
        }
        
        if self.rate_limiting.enabled {
            if self.rate_limiting.standard_rpm == 0 
                || self.rate_limiting.inference_rpm == 0 
                || self.rate_limiting.premium_rpm == 0 {
                return Err(anyhow::anyhow!("Rate limit values must be greater than 0"));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = NativeMessagingConfig::default();
        assert_eq!(config.max_message_size, 1_048_576);
        assert!(config.enable_logging);
        assert_eq!(config.log_level, "info");
        assert!(!config.dev_mode);
        assert!(config.rate_limiting.enabled);
        assert!(config.security.validate_origins);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = NativeMessagingConfig::default();
        assert!(config.validate().is_ok());
        
        // Test invalid max_message_size
        config.max_message_size = 0;
        assert!(config.validate().is_err());
        
        config.max_message_size = 2_000_000; // > 1MB
        assert!(config.validate().is_err());
        
        // Test invalid rate limits
        config.max_message_size = 1_048_576;
        config.rate_limiting.standard_rpm = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_file_operations() -> anyhow::Result<()> {
        let config = NativeMessagingConfig::default();
        
        // Test JSON serialization
        let json_file = NamedTempFile::new()?;
        let json_path = json_file.path().with_extension("json");
        config.to_file(&json_path)?;
        
        let loaded_config = NativeMessagingConfig::from_file(&json_path)?;
        assert_eq!(config.max_message_size, loaded_config.max_message_size);
        assert_eq!(config.enable_logging, loaded_config.enable_logging);
        
        Ok(())
    }
}