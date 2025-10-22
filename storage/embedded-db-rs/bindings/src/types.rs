//! Python wrapper types for core Rust structs

use common::models::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Convert a Rust Node to a Python dictionary
pub fn node_to_dict(py: Python, node: &Node) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    
    match node {
        Node::Chat(chat) => {
            dict.set_item("type", "Chat")?;
            dict.set_item("id", &chat.id)?;
            dict.set_item("title", &chat.title)?;
            dict.set_item("topic", &chat.topic)?;
            dict.set_item("created_at", chat.created_at)?;
            dict.set_item("updated_at", chat.updated_at)?;
            
            // Convert message_ids to Python list
            let msg_ids: Vec<&str> = chat.message_ids.iter().map(|s| s.as_str()).collect();
            dict.set_item("message_ids", msg_ids)?;
            
            // summary_ids (plural)
            let summary_ids: Vec<&str> = chat.summary_ids.iter().map(|s| s.as_str()).collect();
            dict.set_item("summary_ids", summary_ids)?;
            
            if let Some(embedding_id) = &chat.embedding_id {
                dict.set_item("embedding_id", embedding_id)?;
            }
            
            // Metadata as JSON string (Python can parse it)
            let metadata_str = serde_json::to_string(&chat.metadata)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            dict.set_item("metadata", metadata_str)?;
        }
        
        Node::Message(msg) => {
            dict.set_item("type", "Message")?;
            dict.set_item("id", &msg.id)?;
            dict.set_item("chat_id", &msg.chat_id)?;
            dict.set_item("sender", &msg.sender)?;
            dict.set_item("timestamp", msg.timestamp)?;
            dict.set_item("text_content", &msg.text_content)?;
            
            if let Some(embedding_id) = &msg.embedding_id {
                dict.set_item("embedding_id", embedding_id)?;
            }
            
            let metadata_str = serde_json::to_string(&msg.metadata)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            dict.set_item("metadata", metadata_str)?;
        }
        
        Node::Summary(summary) => {
            dict.set_item("type", "Summary")?;
            dict.set_item("id", &summary.id)?;
            dict.set_item("chat_id", &summary.chat_id)?;
            dict.set_item("content", &summary.content)?;
            dict.set_item("created_at", summary.created_at)?;
            
            let msg_ids: Vec<&str> = summary.message_ids.iter().map(|s| s.as_str()).collect();
            dict.set_item("message_ids", msg_ids)?;
            
            if let Some(embedding_id) = &summary.embedding_id {
                dict.set_item("embedding_id", embedding_id)?;
            }
            
            let metadata_str = serde_json::to_string(&summary.metadata)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            dict.set_item("metadata", metadata_str)?;
        }
        
        Node::Entity(entity) => {
            dict.set_item("type", "Entity")?;
            dict.set_item("id", &entity.id)?;
            dict.set_item("label", &entity.label)?;
            dict.set_item("entity_type", &entity.entity_type)?;
            
            if let Some(embedding_id) = &entity.embedding_id {
                dict.set_item("embedding_id", embedding_id)?;
            }
            
            let metadata_str = serde_json::to_string(&entity.metadata)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            dict.set_item("metadata", metadata_str)?;
        }
        
        Node::Attachment(a) => {
            dict.set_item("type", "Attachment")?;
            dict.set_item("id", &a.id)?;
            dict.set_item("message_id", &a.message_id)?;
            dict.set_item("mime_type", &a.mime_type)?;
            dict.set_item("filename", &a.filename)?;
            let metadata_str = serde_json::to_string(&a.metadata)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            dict.set_item("metadata", metadata_str)?;
        }
        
        // Add other node types as needed - just expose basic info for now
        _ => {
            dict.set_item("type", "Other")?;
            dict.set_item("id", node.id())?;
            dict.set_item("note", "Full type conversion not yet implemented")?;
        }
    }
    
    Ok(dict.into())
}

/// Convert a Python dictionary to a Rust Node (simplified for now)
pub fn dict_to_node(dict: &PyDict) -> PyResult<Node> {
    let node_type: String = dict.get_item("type")?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing 'type' field"))?
        .extract()?;
    
    match node_type.as_str() {
        "Chat" => {
            let id: String = dict.get_item("id")?.unwrap().extract()?;
            let title: String = dict.get_item("title")?.unwrap().extract()?;
            let topic: String = dict.get_item("topic")?.unwrap().extract()?;
            let created_at: i64 = dict.get_item("created_at")?.unwrap().extract()?;
            let updated_at: i64 = dict.get_item("updated_at")?.unwrap().extract()?;
            
            let message_ids: Vec<String> = dict.get_item("message_ids")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok(vec![]))?;
            
            let summary_ids: Vec<String> = dict.get_item("summary_ids")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok(vec![]))?;
            
            let embedding_id: Option<String> = dict.get_item("embedding_id")?
                .and_then(|v| v.extract().ok());
            
            let metadata_str: String = dict.get_item("metadata")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok("{}".to_string()))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            Ok(Node::Chat(Chat {
                id,
                title,
                topic,
                created_at,
                updated_at,
                message_ids,
                summary_ids,
                embedding_id,
                metadata,
            }))
        }
        
        "Message" => {
            let id: String = dict.get_item("id")?.unwrap().extract()?;
            let chat_id: String = dict.get_item("chat_id")?.unwrap().extract()?;
            let sender: String = dict.get_item("sender")?.unwrap().extract()?;
            let timestamp: i64 = dict.get_item("timestamp")?.unwrap().extract()?;
            let text_content: String = dict.get_item("text_content")?.unwrap().extract()?;
            
            let attachment_ids: Vec<String> = dict.get_item("attachment_ids")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok(vec![]))?;
            
            let embedding_id: Option<String> = dict.get_item("embedding_id")?
                .and_then(|v| v.extract().ok());
            
            let metadata_str: String = dict.get_item("metadata")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok("{}".to_string()))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            Ok(Node::Message(Message {
                id,
                chat_id,
                sender,
                timestamp,
                text_content,
                attachment_ids,
                embedding_id,
                metadata,
            }))
        }
        
        "Entity" => {
            let id: String = dict.get_item("id")?.unwrap().extract()?;
            let label: String = dict.get_item("label")?.unwrap().extract()?;
            let entity_type: String = dict.get_item("entity_type")?.unwrap().extract()?;
            
            let embedding_id: Option<String> = dict.get_item("embedding_id")?
                .and_then(|v| v.extract().ok());
            
            let metadata_str: String = dict.get_item("metadata")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok("{}".to_string()))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            Ok(Node::Entity(Entity {
                id,
                label,
                entity_type,
                embedding_id,
                metadata,
            }))
        }
        
        "Summary" => {
            let id: String = dict.get_item("id")?.unwrap().extract()?;
            let chat_id: String = dict.get_item("chat_id")?.unwrap().extract()?;
            let content: String = dict.get_item("content")?.unwrap().extract()?;
            let created_at: i64 = dict.get_item("created_at")?.unwrap().extract()?;
            
            let message_ids: Vec<String> = dict.get_item("message_ids")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok(vec![]))?;
            
            let embedding_id: Option<String> = dict.get_item("embedding_id")?
                .and_then(|v| v.extract().ok());
            
            let metadata_str: String = dict.get_item("metadata")?
                .map(|v| v.extract())
                .unwrap_or_else(|| Ok("{}".to_string()))?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            Ok(Node::Summary(Summary {
                id,
                chat_id,
                created_at,
                content,
                message_ids,
                embedding_id,
                metadata,
            }))
        }
        
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unsupported node type for Python->Rust conversion: {}", node_type)
        ))
    }
}

/// Convert a Rust Edge to a Python dictionary
pub fn edge_to_dict(py: Python, edge: &Edge) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("id", &edge.id)?;
    dict.set_item("from_node", &edge.from_node)?;
    dict.set_item("to_node", &edge.to_node)?;
    dict.set_item("edge_type", &edge.edge_type)?;
    dict.set_item("created_at", edge.created_at)?;
    
    let metadata_str = serde_json::to_string(&edge.metadata)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    dict.set_item("metadata", metadata_str)?;
    
    Ok(dict.into())
}

/// Convert a Rust Embedding to a Python dictionary
pub fn embedding_to_dict(py: Python, embedding: &Embedding) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item("id", &embedding.id)?;
    dict.set_item("vector", &embedding.vector)?;
    dict.set_item("model", &embedding.model)?;
    Ok(dict.into())
}
