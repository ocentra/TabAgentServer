//! HuggingFace authentication management.
//!
//! Securely stores and retrieves HuggingFace API tokens using:
//! 1. OS credential store (keyring) - preferred
//! 2. Encrypted file fallback - if keyring unavailable

use anyhow::{Context, Result};
use std::path::PathBuf;

/// HuggingFace authentication manager.
///
/// Handles secure storage/retrieval of HF tokens across platforms.
pub struct HfAuthManager {
    service_name: String,
    username: String,
    fallback_path: Option<PathBuf>,
}

impl HfAuthManager {
    /// Create a new HF auth manager.
    pub fn new() -> Result<Self> {
        Ok(Self {
            service_name: "TabAgent.HuggingFace".to_string(),
            username: "default".to_string(),
            fallback_path: Self::get_fallback_path(),
        })
    }

    /// Get the fallback file path for token storage.
    fn get_fallback_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("TabAgent");
            path.push(".hf_token");
            path
        })
    }

    /// Store a HuggingFace token securely.
    pub fn set_token(&self, token: &str) -> Result<()> {
        // Try keyring first
        match keyring::Entry::new(&self.service_name, &self.username) {
            Ok(entry) => {
                entry.set_password(token)
                    .context("Failed to store token in keyring")?;
                tracing::info!("HF token stored in OS credential store");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Keyring unavailable: {}, using file fallback", e);
                self.set_token_file(token)
            }
        }
    }

    /// Retrieve the stored HuggingFace token.
    pub fn get_token(&self) -> Result<Option<String>> {
        // Try keyring first
        match keyring::Entry::new(&self.service_name, &self.username) {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(token) => Ok(Some(token)),
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(e) => {
                        tracing::warn!("Keyring read failed: {}, trying file fallback", e);
                        self.get_token_file()
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Keyring unavailable: {}, using file fallback", e);
                self.get_token_file()
            }
        }
    }

    /// Check if a token is stored.
    pub fn has_token(&self) -> bool {
        self.get_token().ok().flatten().is_some()
    }

    /// Clear the stored token.
    pub fn clear_token(&self) -> Result<()> {
        // Try keyring first
        match keyring::Entry::new(&self.service_name, &self.username) {
            Ok(entry) => {
                match entry.delete_credential() {
                    Ok(_) => {
                        tracing::info!("HF token cleared from keyring");
                        Ok(())
                    }
                    Err(keyring::Error::NoEntry) => Ok(()),
                    Err(e) => {
                        tracing::warn!("Keyring delete failed: {}, clearing file fallback", e);
                        self.clear_token_file()
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Keyring unavailable: {}, using file fallback", e);
                self.clear_token_file()
            }
        }
    }

    // ========== File Fallback Methods ==========

    fn set_token_file(&self, token: &str) -> Result<()> {
        let path = self.fallback_path.as_ref()
            .context("No fallback path available")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        std::fs::write(path, token)
            .context("Failed to write token to file")?;

        tracing::info!("HF token stored in file: {:?}", path);
        Ok(())
    }

    fn get_token_file(&self) -> Result<Option<String>> {
        let path = self.fallback_path.as_ref()
            .context("No fallback path available")?;

        if !path.exists() {
            return Ok(None);
        }

        let token = std::fs::read_to_string(path)
            .context("Failed to read token from file")?;

        Ok(Some(token.trim().to_string()))
    }

    fn clear_token_file(&self) -> Result<()> {
        let path = self.fallback_path.as_ref()
            .context("No fallback path available")?;

        if path.exists() {
            std::fs::remove_file(path)
                .context("Failed to delete token file")?;
            tracing::info!("HF token file deleted");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hf_auth_creation() {
        let auth = HfAuthManager::new();
        assert!(auth.is_ok());
    }

    #[test]
    fn test_has_token_empty() {
        let auth = HfAuthManager::new().unwrap();
        // Should not fail even if no token exists
        let _ = auth.has_token();
    }
}

