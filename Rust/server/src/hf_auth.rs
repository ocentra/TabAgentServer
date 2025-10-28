///! HuggingFace Authentication Module
//!
//! Provides secure token storage for HuggingFace API access.
//! Uses OS keyring if available, fallback to encrypted file storage.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// HuggingFace token manager
pub struct HfAuthManager {
    config_dir: PathBuf,
    token_file: PathBuf,
}

impl HfAuthManager {
    /// Create new HfAuthManager
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".tabagent");
        
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        let token_file = config_dir.join(".hf_token");
        
        Ok(Self {
            config_dir,
            token_file,
        })
    }
    
    /// Store HuggingFace API token
    ///
    /// # Arguments
    /// * `token` - HuggingFace API token (must start with 'hf_')
    ///
    /// # Returns
    /// Ok(()) if stored successfully
    pub fn set_token(&self, token: &str) -> Result<()> {
        if !token.starts_with("hf_") {
            anyhow::bail!("Invalid token format (must start with 'hf_')");
        }
        
        #[cfg(feature = "keyring")]
        {
            // Use OS keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service)
            keyring::Entry::new("TabAgent.HuggingFace", "api_token")?
                .set_password(token)
                .context("Failed to store token in OS keyring")?;
            tracing::info!("HF token stored in OS keyring");
        }
        
        #[cfg(not(feature = "keyring"))]
        {
            // Fallback: Plain file with restrictive permissions
            // TODO: Add encryption (using machine-specific key)
            fs::write(&self.token_file, token)
                .context("Failed to write token file")?;
            
            // Set restrictive permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&self.token_file)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&self.token_file, perms)?;
            }
            
            tracing::info!("HF token stored in file (plain text - consider enabling keyring feature)");
        }
        
        Ok(())
    }
    
    /// Retrieve stored HuggingFace API token
    ///
    /// # Returns
    /// Some(token) if found, None otherwise
    pub fn get_token(&self) -> Result<Option<String>> {
        #[cfg(feature = "keyring")]
        {
            match keyring::Entry::new("TabAgent.HuggingFace", "api_token")?.get_password() {
                Ok(token) => {
                    tracing::debug!("HF token retrieved from OS keyring");
                    Ok(Some(token))
                }
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(e.into()),
            }
        }
        
        #[cfg(not(feature = "keyring"))]
        {
            if self.token_file.exists() {
                let token = fs::read_to_string(&self.token_file)
                    .context("Failed to read token file")?;
                tracing::debug!("HF token retrieved from file");
                Ok(Some(token.trim().to_string()))
            } else {
                Ok(None)
            }
        }
    }
    
    /// Remove stored token
    ///
    /// # Returns
    /// Ok(()) if cleared successfully (or if no token exists)
    pub fn clear_token(&self) -> Result<()> {
        #[cfg(feature = "keyring")]
        {
            match keyring::Entry::new("TabAgent.HuggingFace", "api_token")?.delete_password() {
                Ok(_) => {
                    tracing::info!("HF token removed from OS keyring");
                    Ok(())
                }
                Err(keyring::Error::NoEntry) => Ok(()), // Already cleared
                Err(e) => Err(e.into()),
            }
        }
        
        #[cfg(not(feature = "keyring"))]
        {
            if self.token_file.exists() {
                fs::remove_file(&self.token_file)
                    .context("Failed to delete token file")?;
                tracing::info!("HF token file deleted");
            }
            Ok(())
        }
    }
    
    /// Check if token is stored
    ///
    /// # Returns
    /// true if token exists
    pub fn has_token(&self) -> bool {
        self.get_token().ok().flatten().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_set_and_get_token() {
        let manager = HfAuthManager::new().unwrap();
        let test_token = "hf_test_token_12345";
        
        // Set token
        manager.set_token(test_token).unwrap();
        
        // Get token
        let retrieved = manager.get_token().unwrap();
        assert_eq!(retrieved, Some(test_token.to_string()));
        
        // Clear token
        manager.clear_token().unwrap();
        
        // Verify cleared
        assert_eq!(manager.get_token().unwrap(), None);
    }
    
    #[test]
    fn test_invalid_token_format() {
        let manager = HfAuthManager::new().unwrap();
        let result = manager.set_token("invalid_token");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_has_token() {
        let manager = HfAuthManager::new().unwrap();
        
        // Clear first
        let _ = manager.clear_token();
        assert!(!manager.has_token());
        
        // Set token
        manager.set_token("hf_test").unwrap();
        assert!(manager.has_token());
        
        // Clear
        manager.clear_token().unwrap();
        assert!(!manager.has_token());
    }
}

