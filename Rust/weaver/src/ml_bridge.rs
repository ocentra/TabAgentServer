//! ML Bridge interface for calling Python ML models.
//!
//! This module defines the trait that abstracts away Python ML function calls.
//! The actual implementation will be in the `ml-bridge` crate using PyO3.

use common::DbResult;

/// Interface for ML model inference functions.
///
/// This trait abstracts the boundary between Rust and Python ML code.
/// Implementations handle the FFI details via PyO3.
#[async_trait::async_trait]
pub trait MlBridge: Send + Sync {
    /// Generate a vector embedding for the given text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to embed
    ///
    /// # Returns
    ///
    /// A vector of floats representing the embedding.
    /// Dimension depends on the model (e.g., 384 for MiniLM, 768 for BERT).
    async fn generate_embedding(&self, text: &str) -> DbResult<Vec<f32>>;

    /// Extract named entities from text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    ///
    /// # Returns
    ///
    /// A list of extracted entities with their labels.
    async fn extract_entities(&self, text: &str) -> DbResult<Vec<Entity>>;

    /// Summarize a conversation.
    ///
    /// # Arguments
    ///
    /// * `messages` - List of message texts to summarize
    ///
    /// # Returns
    ///
    /// A concise summary of the conversation.
    async fn summarize(&self, messages: &[String]) -> DbResult<String>;

    /// Get the current embedding model name.
    ///
    /// # Returns
    ///
    /// The name/identifier of the embedding model currently being used.
    async fn get_embedding_model_name(&self) -> DbResult<String>;

    /// Check if the ML bridge is healthy and responsive.
    async fn health_check(&self) -> DbResult<bool>;
}

/// Represents an extracted entity.
#[derive(Debug, Clone)]
pub struct Entity {
    /// The text span of the entity
    pub text: String,
    
    /// The entity type/label (e.g., "PERSON", "ORG", "GPE")
    pub label: String,
    
    /// Start character position in the original text
    pub start: usize,
    
    /// End character position in the original text
    pub end: usize,
}

/// A mock implementation for testing without Python.
///
/// This allows Rust-only tests and development without requiring
/// the full Python environment.
pub struct MockMlBridge;

#[async_trait::async_trait]
impl MlBridge for MockMlBridge {
    async fn generate_embedding(&self, text: &str) -> DbResult<Vec<f32>> {
        // Generate a simple mock embedding based on text length
        let dim = 384; // Match common embedding dimension
        let mut vec = vec![0.0; dim];
        
        // Simple hash-based mock
        for (i, byte) in text.bytes().enumerate() {
            vec[i % dim] += (byte as f32) / 255.0;
        }
        
        // Normalize
        let magnitude: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut vec {
                *val /= magnitude;
            }
        }
        
        Ok(vec)
    }

    async fn extract_entities(&self, text: &str) -> DbResult<Vec<Entity>> {
        // Mock: Extract capitalized words as entities
        let mut entities = Vec::new();
        let mut start = 0;
        
        for word in text.split_whitespace() {
            if let Some(first_char) = word.chars().next() {
                if first_char.is_uppercase() && word.len() > 2 {
                    entities.push(Entity {
                        text: word.to_string(),
                        label: "MOCK_ENTITY".to_string(),
                        start,
                        end: start + word.len(),
                    });
                }
            }
            start += word.len() + 1;
        }
        
        Ok(entities)
    }

    async fn summarize(&self, messages: &[String]) -> DbResult<String> {
        // Mock: Return first and last message
        if messages.is_empty() {
            return Ok("No messages to summarize.".to_string());
        }
        
        if messages.len() == 1 {
            return Ok(format!("Summary: {}", &messages[0]));
        }
        
        Ok(format!(
            "Conversation starting with '{}' and ending with '{}'.",
            messages.first().unwrap(),
            messages.last().unwrap()
        ))
    }

    async fn get_embedding_model_name(&self) -> DbResult<String> {
        // Mock: Return default model name
        Ok("all-MiniLM-L6-v2".to_string())
    }

    async fn health_check(&self) -> DbResult<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding() {
        let bridge = MockMlBridge;
        let embedding = bridge.generate_embedding("Hello world").await.unwrap();
        
        assert_eq!(embedding.len(), 384);
        
        // Check normalization
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_mock_entity_extraction() {
        let bridge = MockMlBridge;
        let entities = bridge.extract_entities("Alice met Bob in Paris").await.unwrap();
        
        assert_eq!(entities.len(), 3); // Alice, Bob, Paris
        assert_eq!(entities[0].text, "Alice");
        assert_eq!(entities[1].text, "Bob");
        assert_eq!(entities[2].text, "Paris");
    }

    #[tokio::test]
    async fn test_mock_summarize() {
        let bridge = MockMlBridge;
        let messages = vec![
            "Hello there!".to_string(),
            "How are you?".to_string(),
            "Goodbye!".to_string(),
        ];
        
        let summary = bridge.summarize(&messages).await.unwrap();
        assert!(summary.contains("Hello there!"));
        assert!(summary.contains("Goodbye!"));
    }
}

