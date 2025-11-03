//! gRPC service configuration for local vs remote

use std::sync::Arc;
use anyhow::Result;
use tonic::transport::Channel;

use common::grpc::database_service_client::DatabaseServiceClient;
use common::grpc::transformers_service_client::TransformersServiceClient;
use common::grpc::mediapipe_service_client::MediapipeServiceClient;

/// gRPC service endpoints configuration
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Database service endpoint (None = in-process)
    pub database_endpoint: Option<String>,
    
    /// ML inference service endpoint (None = disabled)
    pub ml_endpoint: Option<String>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            database_endpoint: None,  // In-process by default
            ml_endpoint: Some("http://localhost:50051".to_string()),  // Python ML service
        }
    }
}

impl GrpcConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        Self {
            database_endpoint: std::env::var("DATABASE_ENDPOINT").ok(),
            ml_endpoint: std::env::var("ML_ENDPOINT").ok()
                .or_else(|| Some("http://localhost:50051".to_string())),
        }
    }
}

/// Database client - either in-process or remote
pub enum DatabaseClient {
    InProcess(Arc<storage::coordinator::DatabaseCoordinator>),
    Remote(DatabaseServiceClient<Channel>),
}

impl DatabaseClient {
    pub async fn new_in_process() -> Result<Self> {
        let coordinator = storage::coordinator::DatabaseCoordinator::new()?;
        Ok(Self::InProcess(Arc::new(coordinator)))
    }
    
    pub async fn new_remote(endpoint: &str) -> Result<Self> {
        let client = DatabaseServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self::Remote(client))
    }
    
    /// Create from config
    pub async fn from_config(config: &GrpcConfig) -> Result<Self> {
        match &config.database_endpoint {
            Some(endpoint) => {
                tracing::info!("Connecting to remote database: {}", endpoint);
                Self::new_remote(endpoint).await
            }
            None => {
                tracing::info!("Using in-process database");
                Self::new_in_process().await
            }
        }
    }
}

/// ML service clients
pub struct MlClients {
    pub transformers: Option<TransformersServiceClient<Channel>>,
    pub mediapipe: Option<MediapipeServiceClient<Channel>>,
}

impl MlClients {
    pub async fn from_config(config: &GrpcConfig) -> Result<Self> {
        let (transformers, mediapipe) = match &config.ml_endpoint {
            Some(endpoint) => {
                tracing::info!("Connecting to ML service: {}", endpoint);
                let transformers = TransformersServiceClient::connect(endpoint.clone()).await.ok();
                let mediapipe = MediapipeServiceClient::connect(endpoint.clone()).await.ok();
                (transformers, mediapipe)
            }
            None => {
                tracing::info!("ML service disabled");
                (None, None)
            }
        };
        
        Ok(Self { transformers, mediapipe })
    }
}

