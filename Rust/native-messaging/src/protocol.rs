//! Chrome native messaging protocol implementation.
//!
//! This module implements Chrome's native messaging protocol specification
//! for bidirectional communication between Chrome extensions and native applications.

use crate::error::{NativeMessagingError, NativeMessagingResult, ErrorResponse};
use crate::config::NativeMessagingConfig;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt, stdin, stdout};
use std::io::Cursor;

/// Incoming message from Chrome extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    /// Route identifier (e.g., "chat", "embeddings", "health")
    pub route: String,
    
    /// Client-generated request ID for correlation
    pub request_id: String,
    
    /// Route-specific request payload
    pub payload: serde_json::Value,
}

/// Outgoing response to Chrome extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    /// Echo back the client request ID
    pub request_id: String,
    
    /// Indicates success or failure
    pub success: bool,
    
    /// Response data (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    
    /// Error details (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
}

/// Chrome native messaging protocol handler.
pub struct NativeMessagingProtocol {
    config: NativeMessagingConfig,
}

impl NativeMessagingProtocol {
    /// Create a new protocol handler.
    pub fn new(config: NativeMessagingConfig) -> Self {
        Self { config }
    }
    
    /// Read a message from stdin according to Chrome's protocol.
    ///
    /// Chrome's protocol uses a 4-byte little-endian length header
    /// followed by UTF-8 JSON payload.
    ///
    /// # Returns
    ///
    /// * `Ok(IncomingMessage)` - Successfully parsed message
    /// * `Err(NativeMessagingError)` - Protocol error, malformed message, or I/O error
    pub async fn read_message(&self) -> NativeMessagingResult<IncomingMessage> {
        let mut stdin = stdin();
        
        // Read 4-byte length header (little-endian)
        let mut length_bytes = [0u8; 4];
        stdin.read_exact(&mut length_bytes).await
            .map_err(|e| NativeMessagingError::protocol(format!("Failed to read length header: {}", e)))?;
        
        let message_length = u32::from_le_bytes(length_bytes) as usize;
        
        // Validate message length
        if message_length == 0 {
            return Err(NativeMessagingError::protocol("Message length cannot be zero"));
        }
        
        if message_length > self.config.max_message_size {
            return Err(NativeMessagingError::protocol(format!(
                "Message length {} exceeds maximum size {}",
                message_length, self.config.max_message_size
            )));
        }
        
        // Read JSON payload
        let mut message_bytes = vec![0u8; message_length];
        stdin.read_exact(&mut message_bytes).await
            .map_err(|e| NativeMessagingError::protocol(format!("Failed to read message payload: {}", e)))?;
        
        // Parse JSON
        let message_str = String::from_utf8(message_bytes)
            .map_err(|e| NativeMessagingError::protocol(format!("Invalid UTF-8 in message: {}", e)))?;
        
        let message: IncomingMessage = serde_json::from_str(&message_str)
            .map_err(|e| NativeMessagingError::protocol(format!("Invalid JSON in message: {}", e)))?;
        
        // Validate message structure
        self.validate_incoming_message(&message)?;
        
        Ok(message)
    }
    
    /// Write a message to stdout according to Chrome's protocol.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or I/O fails.
    pub async fn write_message(&self, message: &OutgoingMessage) -> NativeMessagingResult<()> {
        let json_value = serde_json::to_value(message)?;
        self.write_raw_message(&json_value).await
    }
    
    /// Write a raw JSON value to stdout.
    ///
    /// # Arguments
    ///
    /// * `message` - The JSON value to send
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or I/O fails.
    pub async fn write_raw_message(&self, message: &serde_json::Value) -> NativeMessagingResult<()> {
        let mut stdout = stdout();
        
        // Serialize to JSON string
        let json_string = serde_json::to_string(message)?;
        let message_bytes = json_string.as_bytes();
        
        // Validate message size
        if message_bytes.len() > self.config.max_message_size {
            return Err(NativeMessagingError::protocol(format!(
                "Response message length {} exceeds maximum size {}",
                message_bytes.len(), self.config.max_message_size
            )));
        }
        
        // Write length header (4-byte little-endian)
        let length = message_bytes.len() as u32;
        let length_bytes = length.to_le_bytes();
        stdout.write_all(&length_bytes).await
            .map_err(|e| NativeMessagingError::protocol(format!("Failed to write length header: {}", e)))?;
        
        // Write JSON payload
        stdout.write_all(message_bytes).await
            .map_err(|e| NativeMessagingError::protocol(format!("Failed to write message payload: {}", e)))?;
        
        // Flush to ensure immediate delivery
        stdout.flush().await
            .map_err(|e| NativeMessagingError::protocol(format!("Failed to flush stdout: {}", e)))?;
        
        Ok(())
    }
    
    /// Parse a message from raw bytes (for testing).
    pub fn parse_message(data: &[u8]) -> NativeMessagingResult<IncomingMessage> {
        if data.len() < 4 {
            return Err(NativeMessagingError::protocol("Data too short for length header"));
        }
        
        let mut cursor = Cursor::new(data);
        let mut length_bytes = [0u8; 4];
        std::io::Read::read_exact(&mut cursor, &mut length_bytes)
            .map_err(|e| NativeMessagingError::protocol(format!("Failed to read length: {}", e)))?;
        
        let message_length = u32::from_le_bytes(length_bytes) as usize;
        
        if data.len() < 4 + message_length {
            return Err(NativeMessagingError::protocol("Data too short for message payload"));
        }
        
        let message_bytes = &data[4..4 + message_length];
        let message_str = String::from_utf8(message_bytes.to_vec())
            .map_err(|e| NativeMessagingError::protocol(format!("Invalid UTF-8: {}", e)))?;
        
        let message: IncomingMessage = serde_json::from_str(&message_str)
            .map_err(|e| NativeMessagingError::protocol(format!("Invalid JSON: {}", e)))?;
        
        Ok(message)
    }
    
    /// Validate incoming message structure.
    fn validate_incoming_message(&self, message: &IncomingMessage) -> NativeMessagingResult<()> {
        if message.route.is_empty() {
            return Err(NativeMessagingError::validation("route", "Route cannot be empty"));
        }
        
        if message.request_id.is_empty() {
            return Err(NativeMessagingError::validation("request_id", "Request ID cannot be empty"));
        }
        
        // Validate route format (alphanumeric, underscore, hyphen only)
        if !message.route.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(NativeMessagingError::validation(
                "route", 
                "Route must contain only alphanumeric characters, underscores, and hyphens"
            ));
        }
        
        Ok(())
    }
}

impl OutgoingMessage {
    /// Create a success response.
    pub fn success(request_id: String, data: serde_json::Value) -> Self {
        Self {
            request_id,
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    /// Create an error response.
    pub fn error(request_id: String, error: ErrorResponse) -> Self {
        Self {
            request_id,
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_parsing() {
        let message = r#"{"route":"health","request_id":"test-123","payload":{}}"#;
        let length = (message.len() as u32).to_le_bytes();
        
        let mut data = Vec::new();
        data.extend_from_slice(&length);
        data.extend_from_slice(message.as_bytes());
        
        let parsed = NativeMessagingProtocol::parse_message(&data).unwrap();
        assert_eq!(parsed.route, "health");
        assert_eq!(parsed.request_id, "test-123");
    }
    
    #[test]
    fn test_message_validation() {
        let protocol = NativeMessagingProtocol::new(NativeMessagingConfig::default());
        
        // Valid message
        let valid_message = IncomingMessage {
            route: "health".to_string(),
            request_id: "test-123".to_string(),
            payload: serde_json::json!({}),
        };
        assert!(protocol.validate_incoming_message(&valid_message).is_ok());
        
        // Empty route
        let invalid_message = IncomingMessage {
            route: "".to_string(),
            request_id: "test-123".to_string(),
            payload: serde_json::json!({}),
        };
        assert!(protocol.validate_incoming_message(&invalid_message).is_err());
        
        // Empty request ID
        let invalid_message = IncomingMessage {
            route: "health".to_string(),
            request_id: "".to_string(),
            payload: serde_json::json!({}),
        };
        assert!(protocol.validate_incoming_message(&invalid_message).is_err());
        
        // Invalid route characters
        let invalid_message = IncomingMessage {
            route: "health/test".to_string(),
            request_id: "test-123".to_string(),
            payload: serde_json::json!({}),
        };
        assert!(protocol.validate_incoming_message(&invalid_message).is_err());
    }
    
    #[test]
    fn test_outgoing_message_creation() {
        let success_msg = OutgoingMessage::success(
            "test-123".to_string(),
            serde_json::json!({"status": "ok"})
        );
        assert!(success_msg.success);
        assert!(success_msg.data.is_some());
        assert!(success_msg.error.is_none());
        
        let error_msg = OutgoingMessage::error(
            "test-456".to_string(),
            ErrorResponse {
                code: "TEST_ERROR".to_string(),
                message: "Test error".to_string(),
                details: None,
                request_id: None,
            }
        );
        assert!(!error_msg.success);
        assert!(error_msg.data.is_none());
        assert!(error_msg.error.is_some());
    }
    
    #[test]
    fn test_protocol_error_handling() {
        // Test short data
        let short_data = vec![1, 2];
        assert!(NativeMessagingProtocol::parse_message(&short_data).is_err());
        
        // Test invalid UTF-8
        let invalid_utf8 = vec![4, 0, 0, 0, 0xFF, 0xFE, 0xFD, 0xFC];
        assert!(NativeMessagingProtocol::parse_message(&invalid_utf8).is_err());
        
        // Test invalid JSON
        let invalid_json_msg = "invalid json";
        let length = (invalid_json_msg.len() as u32).to_le_bytes();
        let mut invalid_json_data = Vec::new();
        invalid_json_data.extend_from_slice(&length);
        invalid_json_data.extend_from_slice(invalid_json_msg.as_bytes());
        assert!(NativeMessagingProtocol::parse_message(&invalid_json_data).is_err());
    }
}