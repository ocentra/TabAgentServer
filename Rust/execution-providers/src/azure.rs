//! Azure Execution Provider
//! 
//! Enables operators that invoke Azure cloud models.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct AzureExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    AzureExecutionProvider,
    "AzureExecutionProvider",
    BackendType::Azure
);

impl AzureExecutionProvider {
    /// Set Azure endpoint URI
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.config.set(ENDPOINT, endpoint);
        self
    }
    
    /// Set authentication token
    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.config.set(AUTH_TOKEN, token);
        self
    }
}

impl ExecutionProvider for AzureExecutionProvider {
    fn name(&self) -> &'static str {
        self.get_name()
    }
    
    fn backend_type(&self) -> BackendType {
        self.get_backend_type()
    }
    
    fn config(&self) -> &ProviderConfig {
        &self.config
    }
    
    fn supported_by_platform(&self) -> bool {
        cfg!(any(target_os = "linux", target_os = "windows", target_os = "android"))
    }
    
    fn is_available(&self) -> Result<bool> {
        // Azure EP requires network connectivity, always return true for platform support
        Ok(self.supported_by_platform())
    }
}

