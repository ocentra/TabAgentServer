//! Knowledge Graph implementation for the MIA system.
//!
//! This crate provides advanced knowledge graph functionality including:
//! - Entity linking and disambiguation
//! - Semantic similarity computation
//! - Graph-based reasoning and inference
//! - Knowledge graph indexing and querying
//!
//! The implementation is inspired by the Weaver modules and follows the MIA
//! cognitive architecture principles.

use common::{DbResult, DbError};
use indexing::IndexManager;
use storage::DatabaseCoordinator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

/// Represents an entity in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier for the entity
    pub id: String,
    
    /// Entity label/name
    pub label: String,
    
    /// Entity type/category
    pub entity_type: String,
    
    /// Entity properties as key-value pairs
    pub properties: HashMap<String, serde_json::Value>,
    
    /// Entity embeddings for semantic similarity
    pub embeddings: Option<Vec<f32>>,
}

/// Represents a relationship between entities in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique identifier for the relationship
    pub id: String,
    
    /// Source entity ID
    pub from_entity: String,
    
    /// Target entity ID
    pub to_entity: String,
    
    /// Relationship type
    pub relationship_type: String,
    
    /// Relationship properties
    pub properties: HashMap<String, serde_json::Value>,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Represents a path between entities in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPath {
    /// Sequence of entity IDs in the path
    pub entities: Vec<String>,
    
    /// Sequence of relationship IDs connecting the entities
    pub relationships: Vec<String>,
    
    /// Total path score
    pub score: f32,
}

/// Knowledge Graph manager that provides entity linking, semantic similarity,
/// and indexing functionality.
pub struct KnowledgeGraph {
    /// Reference to the database coordinator
    _coordinator: DatabaseCoordinator,
    
    /// Reference to the index manager
    _indexing: IndexManager,
    
    /// In-memory entity cache for fast lookups
    entity_cache: HashMap<String, Entity>,
    
    /// In-memory relationship cache for fast lookups
    relationship_cache: HashMap<String, Relationship>,
    
    /// Adjacency list for fast graph traversal
    adjacency_list: HashMap<String, Vec<String>>,
    
    /// Reverse adjacency list for incoming relationships
    reverse_adjacency_list: HashMap<String, Vec<String>>,
}

impl KnowledgeGraph {
    /// Creates a new KnowledgeGraph instance.
    ///
    /// # Arguments
    ///
    /// * `coordinator` - Reference to the database coordinator
    /// * `indexing` - Reference to the index manager
    ///
    /// # Returns
    ///
    /// A new KnowledgeGraph instance
    pub fn new(coordinator: DatabaseCoordinator, indexing: IndexManager) -> Self {
        Self {
            _coordinator: coordinator,
            _indexing: indexing,
            entity_cache: HashMap::new(),
            relationship_cache: HashMap::new(),
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
        }
    }
    
    /// Adds an entity to the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn add_entity(&mut self, entity: Entity) -> DbResult<()> {
        // Add to cache
        self.entity_cache.insert(entity.id.clone(), entity.clone());
        
        // Ensure entity exists in adjacency lists
        self.adjacency_list.entry(entity.id.clone()).or_insert_with(Vec::new);
        self.reverse_adjacency_list.entry(entity.id.clone()).or_insert_with(Vec::new);
        
        // In a real implementation, we would also store the entity in the database
        // and update the indexes
        log::debug!("Added entity {} to knowledge graph", entity.id);
        
        Ok(())
    }
    
    /// Gets an entity from the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The ID of the entity to retrieve
    ///
    /// # Returns
    ///
    /// The entity if found, or None if not found
    pub fn get_entity(&self, entity_id: &str) -> Option<Entity> {
        self.entity_cache.get(entity_id).cloned()
    }
    
    /// Removes an entity from the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The ID of the entity to remove
    ///
    /// # Returns
    ///
    /// True if the entity was found and removed, false otherwise
    pub fn remove_entity(&mut self, entity_id: &str) -> DbResult<bool> {
        let existed = self.entity_cache.remove(entity_id).is_some();
        
        if existed {
            // Remove from adjacency lists
            self.adjacency_list.remove(entity_id);
            self.reverse_adjacency_list.remove(entity_id);
            
            // Remove references to this entity from other entities' adjacency lists
            for neighbors in self.adjacency_list.values_mut() {
                neighbors.retain(|n| n != entity_id);
            }
            
            for neighbors in self.reverse_adjacency_list.values_mut() {
                neighbors.retain(|n| n != entity_id);
            }
            
            // Remove all relationships involving this entity
            self.relationship_cache.retain(|_, rel| {
                rel.from_entity != entity_id && rel.to_entity != entity_id
            });
            
            log::debug!("Removed entity {} from knowledge graph", entity_id);
        }
        
        Ok(existed)
    }
    
    /// Adds a relationship to the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to add
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn add_relationship(&mut self, relationship: Relationship) -> DbResult<()> {
        // Ensure entities exist
        if !self.entity_cache.contains_key(&relationship.from_entity) {
            return Err(DbError::NotFound(format!("Entity {} not found", relationship.from_entity)));
        }
        
        if !self.entity_cache.contains_key(&relationship.to_entity) {
            return Err(DbError::NotFound(format!("Entity {} not found", relationship.to_entity)));
        }
        
        // Add to cache
        self.relationship_cache.insert(relationship.id.clone(), relationship.clone());
        
        // Add to adjacency lists
        self.adjacency_list
            .entry(relationship.from_entity.clone())
            .or_insert_with(Vec::new)
            .push(relationship.to_entity.clone());
            
        self.reverse_adjacency_list
            .entry(relationship.to_entity.clone())
            .or_insert_with(Vec::new)
            .push(relationship.from_entity.clone());
        
        // In a real implementation, we would also store the relationship in the database
        // and update the indexes
        log::debug!("Added relationship {} to knowledge graph", relationship.id);
        
        Ok(())
    }
    
    /// Gets a relationship from the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `relationship_id` - The ID of the relationship to retrieve
    ///
    /// # Returns
    ///
    /// The relationship if found, or None if not found
    pub fn get_relationship(&self, relationship_id: &str) -> Option<Relationship> {
        self.relationship_cache.get(relationship_id).cloned()
    }
    
    /// Removes a relationship from the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `relationship_id` - The ID of the relationship to remove
    ///
    /// # Returns
    ///
    /// True if the relationship was found and removed, false otherwise
    pub fn remove_relationship(&mut self, relationship_id: &str) -> DbResult<bool> {
        if let Some(relationship) = self.relationship_cache.remove(relationship_id) {
            // Remove from adjacency lists
            if let Some(neighbors) = self.adjacency_list.get_mut(&relationship.from_entity) {
                neighbors.retain(|n| n != &relationship.to_entity);
            }
            
            if let Some(neighbors) = self.reverse_adjacency_list.get_mut(&relationship.to_entity) {
                neighbors.retain(|n| n != &relationship.from_entity);
            }
            
            log::debug!("Removed relationship {} from knowledge graph", relationship_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Links entities based on textual content using semantic similarity.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze for entity linking
    /// * `context` - Optional context for disambiguation
    ///
    /// # Returns
    ///
    /// A vector of linked entities with confidence scores
    pub fn link_entities(&self, text: &str, _context: Option<&str>) -> DbResult<Vec<(Entity, f32)>> {
        // In a real implementation, this would use NLP techniques to identify
        // entities in the text and link them to entities in the knowledge graph
        // using semantic similarity and context disambiguation
        
        log::debug!("Linking entities in text: {}", text);
        
        // For now, return an empty result
        Ok(Vec::new())
    }
    
    /// Computes semantic similarity between two entities.
    ///
    /// # Arguments
    ///
    /// * `entity1_id` - The ID of the first entity
    /// * `entity2_id` - The ID of the second entity
    ///
    /// # Returns
    ///
    /// The similarity score (0.0 - 1.0) or an error if entities not found
    pub fn entity_similarity(&self, entity1_id: &str, entity2_id: &str) -> DbResult<f32> {
        let entity1 = self.get_entity(entity1_id)
            .ok_or_else(|| DbError::NotFound(format!("Entity {} not found", entity1_id)))?;
        let entity2 = self.get_entity(entity2_id)
            .ok_or_else(|| DbError::NotFound(format!("Entity {} not found", entity2_id)))?;
        
        // If both entities have embeddings, compute cosine similarity
        if let (Some(embeddings1), Some(embeddings2)) = (&entity1.embeddings, &entity2.embeddings) {
            if embeddings1.len() == embeddings2.len() {
                let dot_product: f32 = embeddings1.iter().zip(embeddings2.iter()).map(|(a, b)| a * b).sum();
                let magnitude1: f32 = embeddings1.iter().map(|x| x * x).sum::<f32>().sqrt();
                let magnitude2: f32 = embeddings2.iter().map(|x| x * x).sum::<f32>().sqrt();
                
                if magnitude1.abs() < f32::EPSILON || magnitude2.abs() < f32::EPSILON {
                    Ok(0.0)
                } else {
                    Ok(dot_product / (magnitude1 * magnitude2))
                }
            } else {
                Err(DbError::InvalidOperation("Entity embeddings have different dimensions".to_string()))
            }
        } else {
            // If no embeddings, compute similarity based on properties
            let similarity = self.compute_property_similarity(&entity1, &entity2);
            Ok(similarity)
        }
    }
    
    /// Computes similarity based on entity properties.
    fn compute_property_similarity(&self, entity1: &Entity, entity2: &Entity) -> f32 {
        // Simple implementation: compare entity types and shared properties
        let type_similarity = if entity1.entity_type == entity2.entity_type { 1.0 } else { 0.0 };
        
        let mut shared_properties = 0;
        let mut total_properties = 0;
        
        for (key, value) in &entity1.properties {
            total_properties += 1;
            if let Some(value2) = entity2.properties.get(key) {
                if value == value2 {
                    shared_properties += 1;
                }
            }
        }
        
        for key in entity2.properties.keys() {
            if !entity1.properties.contains_key(key) {
                total_properties += 1;
            }
        }
        
        let property_similarity = if total_properties > 0 {
            shared_properties as f32 / total_properties as f32
        } else {
            0.0
        };
        
        // Weighted average: 60% type similarity, 40% property similarity
        0.6 * type_similarity + 0.4 * property_similarity
    }
    
    /// Finds related entities based on relationships and semantic similarity.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The ID of the entity to find related entities for
    /// * `max_results` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A vector of related entities with similarity scores
    pub fn find_related_entities(&self, entity_id: &str, _max_results: usize) -> DbResult<Vec<(Entity, f32)>> {
        // In a real implementation, this would traverse the graph to find related entities
        // and compute similarity scores based on relationships and semantic similarity
        
        log::debug!("Finding related entities for: {}", entity_id);
        
        // For now, return an empty result
        Ok(Vec::new())
    }
    
    /// Performs reasoning over the knowledge graph to infer new relationships.
    ///
    /// # Returns
    ///
    /// A vector of inferred relationships
    pub fn perform_reasoning(&self) -> DbResult<Vec<Relationship>> {
        // In a real implementation, this would apply inference rules to discover
        // new relationships based on existing ones
        
        log::debug!("Performing reasoning over knowledge graph");
        
        // For now, return an empty result
        Ok(Vec::new())
    }
    
    /// Gets outgoing neighbors of an entity.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The ID of the entity
    ///
    /// # Returns
    ///
    /// A vector of entity IDs that are directly connected via outgoing relationships
    pub fn get_outgoing_neighbors(&self, entity_id: &str) -> DbResult<Vec<String>> {
        if !self.entity_cache.contains_key(entity_id) {
            return Err(DbError::NotFound(format!("Entity {} not found", entity_id)));
        }
        
        Ok(self.adjacency_list.get(entity_id).cloned().unwrap_or_default())
    }
    
    /// Gets incoming neighbors of an entity.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The ID of the entity
    ///
    /// # Returns
    ///
    /// A vector of entity IDs that are directly connected via incoming relationships
    pub fn get_incoming_neighbors(&self, entity_id: &str) -> DbResult<Vec<String>> {
        if !self.entity_cache.contains_key(entity_id) {
            return Err(DbError::NotFound(format!("Entity {} not found", entity_id)));
        }
        
        Ok(self.reverse_adjacency_list.get(entity_id).cloned().unwrap_or_default())
    }
    
    /// Finds the shortest path between two entities using BFS.
    ///
    /// # Arguments
    ///
    /// * `start_entity_id` - The ID of the start entity
    /// * `end_entity_id` - The ID of the end entity
    ///
    /// # Returns
    ///
    /// An EntityPath representing the shortest path, or None if no path exists
    pub fn find_shortest_path(&self, start_entity_id: &str, end_entity_id: &str) -> DbResult<Option<EntityPath>> {
        if !self.entity_cache.contains_key(start_entity_id) {
            return Err(DbError::NotFound(format!("Entity {} not found", start_entity_id)));
        }
        
        if !self.entity_cache.contains_key(end_entity_id) {
            return Err(DbError::NotFound(format!("Entity {} not found", end_entity_id)));
        }
        
        use std::collections::VecDeque;
        
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessors: HashMap<String, String> = HashMap::new();
        
        queue.push_back(start_entity_id.to_string());
        visited.insert(start_entity_id.to_string());
        
        while let Some(current_entity) = queue.pop_front() {
            if current_entity == end_entity_id {
                // Reconstruct path
                let mut path_entities = Vec::new();
                let mut current = current_entity;
                
                while current != start_entity_id {
                    path_entities.push(current.clone());
                    current = predecessors.get(&current).ok_or_else(|| {
                        DbError::Other("Path reconstruction failed".to_string())
                    })?.clone();
                }
                
                path_entities.push(start_entity_id.to_string());
                path_entities.reverse();
                
                // For now, we don't have relationship IDs, so we'll create empty relationships vector
                return Ok(Some(EntityPath {
                    entities: path_entities,
                    relationships: vec![],
                    score: 0.0,
                }));
            }
            
            // Get neighbors
            if let Some(neighbors) = self.adjacency_list.get(&current_entity) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        predecessors.insert(neighbor.clone(), current_entity.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Performs a semantic search for entities similar to a query entity.
    ///
    /// # Arguments
    ///
    /// * `query_entity_id` - The ID of the query entity
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A vector of (entity_id, similarity_score) tuples, sorted by similarity
    pub fn semantic_search(&self, query_entity_id: &str, limit: usize) -> DbResult<Vec<(String, f32)>> {
        let query_entity = self.get_entity(query_entity_id)
            .ok_or_else(|| DbError::NotFound(format!("Entity {} not found", query_entity_id)))?;
        
        let mut similarities: Vec<(String, f32)> = self.entity_cache
            .iter()
            .filter(|(id, _)| *id != query_entity_id) // Exclude the query entity itself
            .map(|(id, entity)| {
                // Compute similarity with the query entity
                let similarity = if let (Some(query_embeddings), Some(entity_embeddings)) = 
                    (&query_entity.embeddings, &entity.embeddings) {
                    // Use cosine similarity if both have embeddings
                    if query_embeddings.len() == entity_embeddings.len() {
                        let dot_product: f32 = query_embeddings.iter().zip(entity_embeddings.iter()).map(|(a, b)| a * b).sum();
                        let magnitude1: f32 = query_embeddings.iter().map(|x| x * x).sum::<f32>().sqrt();
                        let magnitude2: f32 = entity_embeddings.iter().map(|x| x * x).sum::<f32>().sqrt();
                        
                        if magnitude1.abs() < f32::EPSILON || magnitude2.abs() < f32::EPSILON {
                            0.0
                        } else {
                            dot_product / (magnitude1 * magnitude2)
                        }
                    } else {
                        // Fallback to property similarity if dimensions don't match
                        self.compute_property_similarity(&query_entity, entity)
                    }
                } else {
                    // Use property similarity if one or both don't have embeddings
                    self.compute_property_similarity(&query_entity, entity)
                };
                
                (id.clone(), similarity)
            })
            .collect();
        
        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k results
        similarities.truncate(limit);
        
        Ok(similarities)
    }
    
    /// Gets statistics about the knowledge graph.
    ///
    /// # Returns
    ///
    /// A tuple of (entity_count, relationship_count)
    pub fn get_statistics(&self) -> (usize, usize) {
        (self.entity_cache.len(), self.relationship_cache.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_similarity() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let kg = KnowledgeGraph::new(
            DatabaseCoordinator::with_base_path(Some(temp_dir.path().to_path_buf())).unwrap(),
            IndexManager::new(temp_dir.path().join("index")).unwrap()
        );
        
        let entity1 = Entity {
            id: "entity1".to_string(),
            label: "Apple".to_string(),
            entity_type: "Fruit".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("color".to_string(), serde_json::Value::String("red".to_string()));
                props.insert("weight".to_string(), serde_json::Value::Number(serde_json::Number::from(150)));
                props
            },
            embeddings: Some(vec![0.1, 0.2, 0.3]),
        };
        
        let entity2 = Entity {
            id: "entity2".to_string(),
            label: "Apple".to_string(),
            entity_type: "Fruit".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("color".to_string(), serde_json::Value::String("green".to_string()));
                props.insert("weight".to_string(), serde_json::Value::Number(serde_json::Number::from(140)));
                props
            },
            embeddings: Some(vec![0.1, 0.2, 0.4]),
        };
        
        // For this test, we'll just verify the similarity computation logic
        let similarity = kg.compute_property_similarity(&entity1, &entity2);
        assert!(similarity >= 0.0 && similarity <= 1.0);
    }
    
    #[test]
    fn test_entity_similarity_no_embeddings() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let kg = KnowledgeGraph::new(
            DatabaseCoordinator::with_base_path(Some(temp_dir.path().to_path_buf())).unwrap(),
            IndexManager::new(temp_dir.path().join("index")).unwrap()
        );
        
        let entity1 = Entity {
            id: "entity1".to_string(),
            label: "Apple".to_string(),
            entity_type: "Fruit".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("color".to_string(), serde_json::Value::String("red".to_string()));
                props
            },
            embeddings: None,
        };
        
        let entity2 = Entity {
            id: "entity2".to_string(),
            label: "Orange".to_string(),
            entity_type: "Fruit".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("color".to_string(), serde_json::Value::String("orange".to_string()));
                props
            },
            embeddings: None,
        };
        
        // For this test, we'll just verify the similarity computation logic
        let similarity = kg.compute_property_similarity(&entity1, &entity2);
        assert!(similarity >= 0.0 && similarity <= 1.0);
    }
    
    #[test]
    fn test_graph_operations() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let mut kg = KnowledgeGraph::new(
            DatabaseCoordinator::with_base_path(Some(temp_dir.path().to_path_buf())).unwrap(),
            IndexManager::new(temp_dir.path().join("index")).unwrap()
        );
        
        // Add entities
        let entity1 = Entity {
            id: "entity1".to_string(),
            label: "Entity 1".to_string(),
            entity_type: "Test".to_string(),
            properties: HashMap::new(),
            embeddings: None,
        };
        
        let entity2 = Entity {
            id: "entity2".to_string(),
            label: "Entity 2".to_string(),
            entity_type: "Test".to_string(),
            properties: HashMap::new(),
            embeddings: None,
        };
        
        kg.add_entity(entity1.clone()).unwrap();
        kg.add_entity(entity2.clone()).unwrap();
        
        // Test neighbors (should be empty since no relationships added yet)
        let outgoing = kg.get_outgoing_neighbors("entity1").unwrap();
        assert_eq!(outgoing.len(), 0);
        
        let incoming = kg.get_incoming_neighbors("entity2").unwrap();
        assert_eq!(incoming.len(), 0);
        
        // Test statistics
        let (entity_count, relationship_count) = kg.get_statistics();
        assert_eq!(entity_count, 2);
        assert_eq!(relationship_count, 0);
    }
}