//! ML client for gRPC-based ML inference access
//! 
//! This module provides a unified interface for ML operations (Transformers, Mediapipe)
//! via gRPC to Python services.

use anyhow::Result;
use tonic::transport::Channel;
use common::grpc::ml::{
    transformers_service_client::TransformersServiceClient,
    mediapipe_service_client::MediapipeServiceClient,
    TextRequest, ChatRequest,
};

/// ML client for Transformers and Mediapipe services
#[derive(Clone)]
pub struct MlClient {
    transformers: Option<TransformersServiceClient<Channel>>,
    mediapipe: Option<MediapipeServiceClient<Channel>>,
}

impl MlClient {
    /// Create a new ML client connecting to the specified endpoint
    pub async fn new(endpoint: &str) -> Result<Self> {
        let transformers = TransformersServiceClient::connect(endpoint.to_string()).await.ok();
        let mediapipe = MediapipeServiceClient::connect(endpoint.to_string()).await.ok();
        
        Ok(Self {
            transformers,
            mediapipe,
        })
    }
    
    /// Create a disabled ML client (no services)
    pub fn disabled() -> Self {
        Self {
            transformers: None,
            mediapipe: None,
        }
    }
    
    /// Check if Transformers service is available
    pub fn has_transformers(&self) -> bool {
        self.transformers.is_some()
    }
    
    /// Check if Mediapipe service is available
    pub fn has_mediapipe(&self) -> bool {
        self.mediapipe.is_some()
    }
    
    /// Generate text using Transformers service
    pub async fn generate_text(
        &self,
        prompt: String,
        model: String,
        max_length: i32,
        temperature: f32,
    ) -> Result<Vec<String>> {
        let mut client = self.transformers.clone()
            .ok_or_else(|| anyhow::anyhow!("Transformers service not available"))?;
        
        let request = TextRequest {
            prompt,
            model,
            max_length,
            temperature,
            top_p: 0.9,
        };
        
        let mut stream = client.generate_text(request).await?.into_inner();
        let mut responses = Vec::new();
        
        while let Some(response) = stream.message().await? {
            responses.push(response.text);
            if response.done {
                break;
            }
        }
        
        Ok(responses)
    }
    
    /// Chat completion using Transformers service
    pub async fn chat_completion(
        &self,
        messages: Vec<(String, String)>, // (role, content)
        model: String,
        temperature: f32,
    ) -> Result<String> {
        let mut client = self.transformers.clone()
            .ok_or_else(|| anyhow::anyhow!("Transformers service not available"))?;
        
        let chat_messages: Vec<_> = messages.into_iter()
            .map(|(role, content)| common::grpc::ml::ChatMessage { role, content })
            .collect();
        
        let request = ChatRequest {
            messages: chat_messages,
            model,
            temperature,
        };
        
        let mut stream = client.chat_completion(request).await?.into_inner();
        let mut full_response = String::new();
        
        while let Some(response) = stream.message().await? {
            full_response.push_str(&response.content);
            if response.done {
                break;
            }
        }
        
        Ok(full_response)
    }
}

