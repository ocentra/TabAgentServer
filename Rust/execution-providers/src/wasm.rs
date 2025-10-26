//! WASM Execution Provider
//! 
//! WebAssembly execution provider for browser environments.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};

#[derive(Debug, Clone)]
pub struct WASMExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    WASMExecutionProvider,
    "WASMExecutionProvider",
    BackendType::WASM
);

impl WASMExecutionProvider {}

impl ExecutionProvider for WASMExecutionProvider {
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
        cfg!(target_arch = "wasm32")
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(true) // Always available in WASM
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        Ok(false)
    }
}

