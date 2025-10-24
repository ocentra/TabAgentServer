//! Data models for the storage layer.
//!
//! This module defines all core data structures using the Hybrid Schema Model:
//! - Strongly-typed fields for queryable, critical data
//! - Flexible `metadata` field for extensibility

use crate::{EdgeId, EmbeddingId, NodeId};
use serde::{Deserialize, Serialize};

// --- Node Enum ---

/// The unifying enum for all types of nodes in the knowledge graph.
///
/// This allows storing different node types in the same sled Tree while
/// maintaining type safety through Rust's enum system.
///
/// # Examples
///
/// ```
/// use common::models::{Node, Chat};
/// use common::NodeId;
/// use serde_json::json;
///
/// let chat = Chat {
///     id: NodeId::new("chat_123"),
///     title: "Project Discussion".to_string(),
///     topic: "Rust Database".to_string(),
///     created_at: 1697500000000,
///     updated_at: 1697500000000,
///     message_ids: vec![],
///     summary_ids: vec![],
///     embedding_id: None,
///     metadata: json!({}),
/// };
///
/// let node = Node::Chat(chat);
/// assert_eq!(node.id().as_str(), "chat_123");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Node {
    /// A conversation or chat session.
    Chat(Chat),
    /// A message within a chat.
    Message(Message),
    /// A summary of a conversation or section.
    Summary(Summary),
    /// An attached file or media.
    Attachment(Attachment),
    /// An extracted entity (person, project, concept, etc.).
    Entity(Entity),
    /// A web search query and its results.
    WebSearch(WebSearch),
    /// A scraped web page.
    ScrapedPage(ScrapedPage),
    /// A user bookmark.
    Bookmark(Bookmark),
    /// Metadata for an image (OCR, objects, faces).
    ImageMetadata(ImageMetadata),
    /// Transcribed audio content.
    AudioTranscript(AudioTranscript),
    /// AI model information.
    ModelInfo(ModelInfo),
    /// Agent action outcome for learning and improvement.
    ActionOutcome(ActionOutcome),
}

impl Node {
    /// Returns the ID of the node regardless of its variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use common::models::{Node, Entity};
    /// use common::NodeId;
    /// use serde_json::json;
    ///
    /// let entity = Entity {
    ///     id: NodeId::new("entity_456"),
    ///     label: "Project Phoenix".to_string(),
    ///     entity_type: "PROJECT".to_string(),
    ///     embedding_id: None,
    ///     metadata: json!({}),
    /// };
    ///
    /// let node = Node::Entity(entity);
    /// assert_eq!(node.id().as_str(), "entity_456");
    /// ```
    #[inline]
    pub fn id(&self) -> &NodeId {
        match self {
            Node::Chat(c) => &c.id,
            Node::Message(m) => &m.id,
            Node::Summary(s) => &s.id,
            Node::Attachment(a) => &a.id,
            Node::Entity(e) => &e.id,
            Node::WebSearch(w) => &w.id,
            Node::ScrapedPage(s) => &s.id,
            Node::Bookmark(b) => &b.id,
            Node::ImageMetadata(i) => &i.id,
            Node::AudioTranscript(a) => &a.id,
            Node::ModelInfo(m) => &m.id,
            Node::ActionOutcome(a) => &a.id,
        }
    }
}

// --- Concrete Node Types ---

/// A conversation or chat session.
///
/// Represents a top-level container for messages and summaries.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    // --- Core, Indexed Fields ---
    /// Unique identifier for this chat.
    pub id: NodeId,
    /// User-facing title of the chat.
    pub title: String,
    /// Topic or subject, populated by the Knowledge Weaver.
    pub topic: String,
    /// Unix timestamp (milliseconds) when the chat was created.
    pub created_at: i64,
    /// Unix timestamp (milliseconds) when the chat was last updated.
    pub updated_at: i64,

    // --- Core, Unindexed Fields ---
    /// IDs of messages belonging to this chat.
    pub message_ids: Vec<NodeId>,
    /// IDs of summaries for this chat.
    pub summary_ids: Vec<NodeId>,
    /// Optional reference to the chat's embedding.
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// A message within a chat.
///
/// Represents a single message from a user or assistant.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    // --- Core, Indexed Fields ---
    /// Unique identifier for this message.
    pub id: NodeId,
    /// ID of the chat this message belongs to.
    pub chat_id: NodeId,
    /// Sender identifier (e.g., "user", "assistant", user ID).
    pub sender: String,
    /// Unix timestamp (milliseconds) when the message was sent.
    pub timestamp: i64,

    // --- Core, Unindexed Fields ---
    /// The text content of the message.
    pub text_content: String,
    /// IDs of attachments linked to this message.
    pub attachment_ids: Vec<NodeId>,
    /// Optional reference to the message's embedding.
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// An extracted entity (person, project, concept, etc.).
///
/// Entities are identified through Named Entity Recognition (NER)
/// and linked across conversations.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entity {
    // --- Core, Indexed Fields ---
    /// Unique identifier for this entity.
    pub id: NodeId,
    /// The canonical name of the entity (e.g., "Project Phoenix").
    pub label: String,
    /// Type of entity (e.g., "PERSON", "PROJECT", "CONCEPT").
    pub entity_type: String,

    // --- Core, Unindexed Fields ---
    /// Optional reference to the entity's embedding.
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// A summary of a conversation or section.
///
/// Generated periodically by the Knowledge Weaver to consolidate
/// information from multiple messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Summary {
    // --- Core, Indexed Fields ---
    /// Unique identifier for this summary.
    pub id: NodeId,
    /// ID of the chat this summary belongs to.
    pub chat_id: NodeId,
    /// Unix timestamp (milliseconds) when the summary was created.
    pub created_at: i64,

    // --- Core, Unindexed Fields ---
    /// The summary text content.
    pub content: String,
    /// IDs of messages covered by this summary.
    pub message_ids: Vec<NodeId>,
    /// Optional reference to the summary's embedding.
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// An attached file or media.
///
/// Represents files, images, audio, or other binary content
/// associated with messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attachment {
    // --- Core, Indexed Fields ---
    /// Unique identifier for this attachment.
    pub id: NodeId,
    /// ID of the message this attachment belongs to.
    pub message_id: NodeId,
    /// MIME type of the attachment (e.g., "image/png").
    pub mime_type: String,
    /// Unix timestamp (milliseconds) when the attachment was created.
    pub created_at: i64,

    // --- Core, Unindexed Fields ---
    /// Original filename of the attachment.
    pub filename: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Storage path or URL where the file is located.
    pub storage_path: String,
    /// Extracted text from OCR (images) or transcription (audio/video).
    pub extracted_text: Option<String>,
    /// Detected objects in images (e.g., ["cat", "laptop"]).
    pub detected_objects: Vec<String>,
    /// Optional reference to the attachment's embedding (for images, etc.).
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

// --- Edge ---

/// A directed, typed relationship between two nodes.
///
/// Edges represent relationships in the knowledge graph, such as
/// "CONTAINS_MESSAGE", "MENTIONS_ENTITY", etc.
///
/// # Examples
///
/// ```
/// use common::models::Edge;
/// use common::{EdgeId, NodeId};
/// use serde_json::json;
///
/// let edge = Edge {
///     id: EdgeId::new("edge_789"),
///     from_node: NodeId::new("message_123"),
///     to_node: NodeId::new("entity_456"),
///     edge_type: "MENTIONS".to_string(),
///     created_at: 1697500000000,
///     metadata: json!({"confidence": 0.95}),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    /// Unique identifier for this edge.
    pub id: EdgeId,
    /// ID of the source node.
    pub from_node: NodeId,
    /// ID of the target node.
    pub to_node: NodeId,
    /// Type of relationship (e.g., "CONTAINS_MESSAGE", "MENTIONS").
    pub edge_type: String,
    /// Unix timestamp (milliseconds) when the edge was created.
    pub created_at: i64,
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

// --- Embedding ---

// --- MIA-Specific Node Types ---

/// A web search query and its results.
///
/// Tracks user searches for context and learning.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSearch {
    /// Unique identifier for this web search.
    pub id: NodeId,
    /// The search query text.
    pub query: String,
    /// Unix timestamp (milliseconds) when the search was performed.
    pub timestamp: i64,
    /// URLs of the search results.
    pub results_urls: Vec<String>,
    /// Optional reference to the query's embedding.
    pub embedding_id: Option<EmbeddingId>,
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// A scraped web page.
///
/// Stores content from web pages visited or scraped by the user.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScrapedPage {
    /// Unique identifier for this scraped page.
    pub id: NodeId,
    /// The URL of the scraped page.
    pub url: String,
    /// Unix timestamp (milliseconds) when the page was scraped.
    pub scraped_at: i64,
    /// Hash of the content for change detection.
    pub content_hash: String,
    /// Page title (if available).
    pub title: Option<String>,
    /// Extracted text content from the page.
    pub text_content: String,
    /// Optional reference to the page's embedding.
    pub embedding_id: Option<EmbeddingId>,
    /// Storage path for full HTML or archived content.
    pub storage_path: String,
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// A user bookmark.
///
/// Represents a saved URL with user annotations.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bookmark {
    /// Unique identifier for this bookmark.
    pub id: NodeId,
    /// The bookmarked URL.
    pub url: String,
    /// User-provided title or auto-detected page title.
    pub title: String,
    /// User's notes or description.
    pub description: Option<String>,
    /// Unix timestamp (milliseconds) when the bookmark was created.
    pub created_at: i64,
    /// Tags for organization.
    pub tags: Vec<String>,
    /// Optional reference to the bookmark's embedding.
    pub embedding_id: Option<EmbeddingId>,
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// Metadata extracted from an image.
///
/// Stores OCR, object detection, and face recognition results.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageMetadata {
    /// Unique identifier for this image metadata.
    pub id: NodeId,
    /// File path to the image.
    pub file_path: String,
    /// Detected objects in the image (e.g., ["cat", "laptop", "desk"]).
    pub detected_objects: Vec<String>,
    /// Detected faces (could be identifiers or descriptions).
    pub detected_faces: Vec<String>,
    /// Text extracted via OCR.
    pub ocr_text: Option<String>,
    /// Optional reference to the image's embedding (visual features).
    pub embedding_id: Option<EmbeddingId>,
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// Transcribed audio content.
///
/// Stores transcriptions from audio files or voice messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioTranscript {
    /// Unique identifier for this audio transcript.
    pub id: NodeId,
    /// File path to the audio file.
    pub file_path: String,
    /// Unix timestamp (milliseconds) when the transcription was created.
    pub transcribed_at: i64,
    /// The transcribed text.
    pub transcript: String,
    /// Speaker diarization data (JSON string with speaker segments).
    pub speaker_diarization: Option<String>,
    /// Optional reference to the transcript's embedding.
    pub embedding_id: Option<EmbeddingId>,
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// AI model information.
///
/// Tracks loaded AI models for MIA's multi-model capabilities.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelInfo {
    /// Unique identifier for this model.
    pub id: NodeId,
    /// Model name (e.g., "Llama-3.2-1B").
    pub name: String,
    /// File path to the model.
    pub path: String,
    /// Model file size in bytes.
    pub size_bytes: u64,
    /// SHA-256 hash for integrity verification.
    pub sha256: String,
    /// Model format (e.g., "GGUF", "ONNX", "SafeTensors").
    pub format: String,
    /// Unix timestamp (milliseconds) when the model was last loaded.
    pub loaded_at: Option<i64>,
    /// Application-specific metadata stored as JSON.
    #[serde(with = "crate::json_metadata")]
    pub metadata: serde_json::Value,
}

/// Agent action outcome for learning and improvement.
///
/// Records the results of agent actions for experience-based learning.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionOutcome {
    /// Unique identifier for this action outcome.
    pub id: NodeId,
    /// Type of action performed (e.g., "search", "scrape", "summarize").
    pub action_type: String,
    /// Arguments passed to the action as JSON.
    #[serde(with = "crate::json_metadata")]
    pub action_args: serde_json::Value,
    /// Result of the action (success/failure and returned data).
    #[serde(with = "crate::json_metadata")]
    pub result: serde_json::Value,
    /// Optional user feedback on the action.
    pub user_feedback: Option<UserFeedback>,
    /// Unix timestamp (milliseconds) when the action was performed.
    pub timestamp: i64,
    /// Context where the action occurred (message ID or conversation context).
    pub conversation_context: String,
}

/// User feedback on agent actions.
///
/// Captures user approval, correction, or rejection of agent behavior.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserFeedback {
    /// Type of feedback provided.
    pub feedback_type: FeedbackType,
    /// Optional user comment explaining the feedback.
    pub user_comment: Option<String>,
    /// Optional correction provided by the user.
    pub correction: Option<String>,
    /// Unix timestamp (milliseconds) when the feedback was provided.
    pub timestamp: i64,
}

/// Types of user feedback.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FeedbackType {
    /// User corrected the agent's action.
    Correction,
    /// User approved the agent's action.
    Approval,
    /// User rejected the agent's action.
    Rejection,
    /// No explicit feedback (infer from follow-up or lack of response).
    Neutral,
}

/// A high-dimensional vector embedding.
///
/// Used for semantic search and similarity comparisons.
///
/// # Examples
///
/// ```
/// use common::models::Embedding;
/// use common::EmbeddingId;
///
/// let embedding = Embedding {
///     id: EmbeddingId::new("embed_001"),
///     vector: vec![0.1, 0.2, 0.3], // Typically 384 or 768 dimensions
///     model: "all-MiniLM-L6-v2".to_string(),
/// };
///
/// assert_eq!(embedding.vector.len(), 3);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Embedding {
    /// Unique identifier for this embedding.
    pub id: EmbeddingId,
    /// The vector as a list of f32 values.
    pub vector: Vec<f32>,
    /// The model used to generate this embedding.
    pub model: String,
}

