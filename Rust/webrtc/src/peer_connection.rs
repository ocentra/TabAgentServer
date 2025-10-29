//! Real WebRTC peer connection handler using webrtc crate

use crate::{
    config::WebRtcConfig,
    data_channel::DataChannelHandler,
    error::{WebRtcError, WebRtcResult},
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use webrtc::{
    api::{
        media_engine::MediaEngine,
        setting_engine::SettingEngine,
        APIBuilder,
    },
    data_channel::{
        data_channel_message::DataChannelMessage,
        RTCDataChannel,
    },
    ice_transport::{
        ice_candidate::RTCIceCandidateInit,
        ice_server::RTCIceServer,
    },
    peer_connection::{
        configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
};

/// Real WebRTC peer connection wrapper
pub struct PeerConnectionHandler {
    /// Underlying WebRTC peer connection
    peer_connection: Arc<RTCPeerConnection>,
    
    /// Data channel for bidirectional communication
    data_channel: Arc<RwLock<Option<Arc<RTCDataChannel>>>>,
    
    /// Handler for data channel messages
    data_channel_handler: Arc<DataChannelHandler>,
    
    /// AppState handler for routing messages
    app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, Result<tabagent_values::ResponseValue>> + Send + Sync>,
}

impl PeerConnectionHandler {
    /// Create a new peer connection handler
    pub async fn new(
        config: &WebRtcConfig,
        app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, Result<tabagent_values::ResponseValue>> + Send + Sync>,
    ) -> WebRtcResult<Self> {
        // Create WebRTC API
        let mut media_engine = MediaEngine::default();
        
        // We only need data channels, not media
        media_engine.register_default_codecs().map_err(|e| {
            WebRtcError::InternalError(format!("Failed to register codecs: {}", e))
        })?;
        
        let setting_engine = SettingEngine::default();
        
        // Note: ICE timeouts can be configured if needed in the future
        
        // Create and configure interceptor registry
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_setting_engine(setting_engine)
            .build();
        
        // Create ICE servers from config
        let mut ice_servers = Vec::new();
        
        // Add STUN servers
        for stun_url in &config.stun_servers {
            ice_servers.push(RTCIceServer {
                urls: vec![stun_url.clone()],
                ..Default::default()
            });
        }
        
        // Add TURN servers
        for turn_server in &config.turn_servers {
            ice_servers.push(RTCIceServer {
                urls: turn_server.urls.clone(),
                username: turn_server.username.clone(),
                credential: turn_server.credential.clone(),
                ..Default::default()
            });
        }
        
        let rtc_config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };
        
        // Create peer connection
        let peer_connection = api.new_peer_connection(rtc_config).await.map_err(|e| {
            WebRtcError::InternalError(format!("Failed to create peer connection: {}", e))
        })?;
        
        let data_channel_handler = Arc::new(DataChannelHandler::new(config.max_message_size));
        
        Ok(Self {
            peer_connection: Arc::new(peer_connection),
            data_channel: Arc::new(RwLock::new(None)),
            data_channel_handler,
            app_state_handler,
        })
    }
    
    /// Create a data channel and generate SDP offer
    pub async fn create_offer(&self) -> WebRtcResult<String> {
        // Create data channel
        let data_channel = self.peer_connection
            .create_data_channel("tabagent", None)
            .await
            .map_err(|e| WebRtcError::InternalError(format!("Failed to create data channel: {}", e)))?;
        
        // data_channel is already Arc from webrtc crate
        // Set up data channel event handlers
        let dc_clone = data_channel.clone();
        let handler_clone = self.data_channel_handler.clone();
        let app_state_clone = self.app_state_handler.clone();
        
        data_channel.on_open(Box::new(move || {
            tracing::info!("Data channel opened");
            Box::pin(async {})
        }));
        
        data_channel.on_close(Box::new(move || {
            tracing::info!("Data channel closed");
            Box::pin(async {})
        }));
        
        data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
            let handler = handler_clone.clone();
            let app_state = app_state_clone.clone();
            let dc = dc_clone.clone();
            
            Box::pin(async move {
                tracing::debug!("Received data channel message: {} bytes", msg.data.len());
                
                // Process message through handler
                let response_bytes = handler.handle_message_safe(
                    &msg.data,
                    |request| {
                        let app_state = app_state.clone();
                        async move {
                            app_state(request).await
                        }
                    }
                ).await;
                
                // Send response back
                let response_msg = bytes::Bytes::copy_from_slice(&response_bytes);
                if let Err(e) = dc.send(&response_msg).await {
                    tracing::error!("Failed to send response: {}", e);
                }
            })
        }));
        
        // Store data channel reference (already Arc from webrtc crate)
        *self.data_channel.write().await = Some(data_channel);
        
        // Create offer
        let offer = self.peer_connection.create_offer(None).await.map_err(|e| {
            WebRtcError::InternalError(format!("Failed to create offer: {}", e))
        })?;
        
        // Set local description
        self.peer_connection.set_local_description(offer.clone()).await.map_err(|e| {
            WebRtcError::InternalError(format!("Failed to set local description: {}", e))
        })?;
        
        Ok(offer.sdp)
    }
    
    /// Process remote SDP answer
    pub async fn set_answer(&self, sdp: String) -> WebRtcResult<()> {
        let answer = RTCSessionDescription::answer(sdp).map_err(|e| {
            WebRtcError::BadRequest(format!("Invalid SDP answer: {}", e))
        })?;
        
        self.peer_connection.set_remote_description(answer).await.map_err(|e| {
            WebRtcError::InternalError(format!("Failed to set remote description: {}", e))
        })?;
        
        Ok(())
    }
    
    /// Add ICE candidate
    pub async fn add_ice_candidate(&self, candidate: String) -> WebRtcResult<()> {
        let ice_candidate = RTCIceCandidateInit {
            candidate,
            ..Default::default()
        };
        
        self.peer_connection.add_ice_candidate(ice_candidate).await.map_err(|e| {
            WebRtcError::InternalError(format!("Failed to add ICE candidate: {}", e))
        })?;
        
        Ok(())
    }
    
    /// Get connection state
    pub fn connection_state(&self) -> RTCPeerConnectionState {
        self.peer_connection.connection_state()
    }
    
    /// Close the peer connection
    pub async fn close(&self) -> WebRtcResult<()> {
        self.peer_connection.close().await.map_err(|e| {
            WebRtcError::InternalError(format!("Failed to close peer connection: {}", e))
        })?;
        Ok(())
    }
    
    /// Send message over data channel
    pub async fn send_message(&self, data: &[u8]) -> WebRtcResult<()> {
        let dc_guard = self.data_channel.read().await;
        let dc = dc_guard.as_ref()
            .ok_or_else(|| WebRtcError::InternalError("Data channel not initialized".to_string()))?;
        
        let msg = bytes::Bytes::copy_from_slice(data);
        dc.send(&msg).await.map_err(|e| {
            WebRtcError::InternalError(format!("Failed to send message: {}", e))
        })?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
    
    #[tokio::test]
    async fn test_peer_connection_creation() {
        let config = WebRtcConfig::default();
        
        let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
            Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
        
        let pc_handler = PeerConnectionHandler::new(&config, handler).await;
        assert!(pc_handler.is_ok(), "Should create peer connection");
    }
    
    #[tokio::test]
    async fn test_create_offer() {
        let config = WebRtcConfig::default();
        
        let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
            Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
        
        let pc_handler = PeerConnectionHandler::new(&config, handler).await.unwrap();
        let offer_sdp = pc_handler.create_offer().await;
        
        assert!(offer_sdp.is_ok(), "Should generate offer SDP");
        let sdp = offer_sdp.unwrap();
        assert!(sdp.contains("v=0"), "SDP should be valid");
        assert!(sdp.contains("application"), "SDP should include data channel");
    }
}

