//! Payload indexing and filtering for hybrid search.
//!
//! This module provides functionality for indexing and filtering vector payloads
//! (metadata associated with vectors) to enable hybrid search capabilities.
//! It follows the Rust Architecture Guidelines for safety, performance, and clarity.

use common::{DbError, DbResult, EmbeddingId};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use ordered_float::OrderedFloat;

/// A trait for types that can be used as payload field values.
pub trait PayloadValue: Clone + Debug + PartialEq + Eq + Hash + Send + Sync {}
impl<T: Clone + Debug + PartialEq + Eq + Hash + Send + Sync> PayloadValue for T {}

/// A field in a payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayloadField {
    /// The name of the field.
    pub name: String,
    
    /// The value of the field.
    pub value: PayloadFieldValue,
}

/// A value in a payload field.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PayloadFieldValue {
    /// A string value.
    String(String),
    
    /// An integer value.
    Integer(i64),
    
    /// A floating-point value.
    Float(OrderedFloat<f64>),
    
    /// A boolean value.
    Boolean(bool),
    
    /// A list of values.
    List(Vec<PayloadFieldValue>),
    
    /// A geo point value.
    GeoPoint { lat: OrderedFloat<f64>, lon: OrderedFloat<f64> },
}

impl PayloadFieldValue {
    /// Checks if this value matches the given filter condition.
    pub fn matches(&self, condition: &PayloadCondition) -> bool {
        match condition {
            PayloadCondition::Match { value } => self == value,
            PayloadCondition::Range { from, to } => {
                match self {
                    PayloadFieldValue::Integer(i) => {
                        let from_match = from.map_or(true, |f| *i >= f.into_inner() as i64);
                        let to_match = to.map_or(true, |t| *i <= t.into_inner() as i64);
                        from_match && to_match
                    }
                    PayloadFieldValue::Float(f) => {
                        let from_match = from.map_or(true, |from_val| **f >= from_val.into_inner());
                        let to_match = to.map_or(true, |to_val| **f <= to_val.into_inner());
                        from_match && to_match
                    }
                    _ => false,
                }
            }
            PayloadCondition::GeoRadius { center, radius } => {
                match self {
                    PayloadFieldValue::GeoPoint { lat, lon } => {
                        let distance = haversine_distance(lat.into_inner(), lon.into_inner(), center.lat.into_inner(), center.lon.into_inner());
                        distance <= radius.into_inner()
                    }
                    _ => false,
                }
            }
        }
    }
}

/// Calculates the haversine distance between two geo points in meters.
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371000.0; // Earth radius in meters
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

/// A geographic point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoPoint {
    /// Latitude in degrees.
    pub lat: OrderedFloat<f64>,
    
    /// Longitude in degrees.
    pub lon: OrderedFloat<f64>,
}

/// A condition for filtering payload fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayloadCondition {
    /// Matches a specific value.
    Match { value: PayloadFieldValue },
    
    /// Matches a range of values.
    Range { from: Option<OrderedFloat<f64>>, to: Option<OrderedFloat<f64>> },
    
    /// Matches points within a geographic radius.
    GeoRadius { center: GeoPoint, radius: OrderedFloat<f64> },
}

/// A filter for payload fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadFilter {
    /// Conditions that must all be satisfied (AND).
    pub must: Vec<PayloadCondition>,
    
    /// Conditions where at least one must be satisfied (OR).
    pub should: Vec<PayloadCondition>,
    
    /// Conditions that must not be satisfied (NOT).
    pub must_not: Vec<PayloadCondition>,
}

impl PayloadFilter {
    /// Creates a new empty filter.
    pub fn new() -> Self {
        Self {
            must: Vec::new(),
            should: Vec::new(),
            must_not: Vec::new(),
        }
    }
    
    /// Adds a condition that must be satisfied.
    pub fn must(mut self, condition: PayloadCondition) -> Self {
        self.must.push(condition);
        self
    }
    
    /// Adds a condition where at least one must be satisfied.
    pub fn should(mut self, condition: PayloadCondition) -> Self {
        self.should.push(condition);
        self
    }
    
    /// Adds a condition that must not be satisfied.
    pub fn must_not(mut self, condition: PayloadCondition) -> Self {
        self.must_not.push(condition);
        self
    }
    
    /// Checks if a payload satisfies this filter.
    pub fn matches(&self, payload: &Payload) -> bool {
        // Check must conditions
        for condition in &self.must {
            if !payload.matches_condition(condition) {
                return false;
            }
        }
        
        // Check must_not conditions
        for condition in &self.must_not {
            if payload.matches_condition(condition) {
                return false;
            }
        }
        
        // Check should conditions
        if !self.should.is_empty() {
            let mut any_match = false;
            for condition in &self.should {
                if payload.matches_condition(condition) {
                    any_match = true;
                    break;
                }
            }
            if !any_match {
                return false;
            }
        }
        
        true
    }
}

impl Default for PayloadFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// A payload (metadata) associated with a vector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payload {
    /// The fields in the payload.
    pub fields: BTreeMap<String, PayloadFieldValue>,
}

impl Payload {
    /// Creates a new empty payload.
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }
    
    /// Adds a field to the payload.
    pub fn add_field(&mut self, name: String, value: PayloadFieldValue) {
        self.fields.insert(name, value);
    }
    
    /// Gets a field from the payload.
    pub fn get_field(&self, name: &str) -> Option<&PayloadFieldValue> {
        self.fields.get(name)
    }
    
    /// Removes a field from the payload.
    pub fn remove_field(&mut self, name: &str) -> Option<PayloadFieldValue> {
        self.fields.remove(name)
    }
    
    /// Checks if the payload matches a condition.
    pub fn matches_condition(&self, condition: &PayloadCondition) -> bool {
        // For simplicity, we'll check if any field matches the condition
        // In a real implementation, we would need to specify which field to check
        self.fields.values().any(|value| value.matches(condition))
    }
    
    /// Converts from a serde_json::Value.
    pub fn from_json_value(value: JsonValue) -> DbResult<Self> {
        let mut payload = Self::new();
        
        if let JsonValue::Object(map) = value {
            for (key, val) in map {
                let field_value = match val {
                    JsonValue::String(s) => PayloadFieldValue::String(s),
                    JsonValue::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            PayloadFieldValue::Integer(i)
                        } else if let Some(f) = n.as_f64() {
                            PayloadFieldValue::Float(OrderedFloat(f))
                        } else {
                            return Err(DbError::InvalidOperation("Invalid number format".to_string()));
                        }
                    }
                    JsonValue::Bool(b) => PayloadFieldValue::Boolean(b),
                    JsonValue::Array(arr) => {
                        let mut list = Vec::new();
                        for item in arr {
                            list.push(Self::json_value_to_field_value(item)?);
                        }
                        PayloadFieldValue::List(list)
                    }
                    JsonValue::Object(obj) => {
                        // Check if it's a geo point
                        if obj.contains_key("lat") && obj.contains_key("lon") {
                            let lat = obj.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let lon = obj.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            PayloadFieldValue::GeoPoint { 
                                lat: OrderedFloat(lat), 
                                lon: OrderedFloat(lon) 
                            }
                        } else {
                            return Err(DbError::InvalidOperation("Unsupported object format".to_string()));
                        }
                    }
                    JsonValue::Null => {
                        return Err(DbError::InvalidOperation("Null values not supported".to_string()));
                    }
                };
                payload.add_field(key, field_value);
            }
        } else {
            return Err(DbError::InvalidOperation("Payload must be an object".to_string()));
        }
        
        Ok(payload)
    }
    
    /// Converts a serde_json::Value to a PayloadFieldValue.
    fn json_value_to_field_value(value: JsonValue) -> DbResult<PayloadFieldValue> {
        match value {
            JsonValue::String(s) => Ok(PayloadFieldValue::String(s)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(PayloadFieldValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(PayloadFieldValue::Float(OrderedFloat(f)))
                } else {
                    Err(DbError::InvalidOperation("Invalid number format".to_string()))
                }
            }
            JsonValue::Bool(b) => Ok(PayloadFieldValue::Boolean(b)),
            JsonValue::Array(arr) => {
                let mut list = Vec::new();
                for item in arr {
                    list.push(Self::json_value_to_field_value(item)?);
                }
                Ok(PayloadFieldValue::List(list))
            }
            JsonValue::Object(obj) => {
                // Check if it's a geo point
                if obj.contains_key("lat") && obj.contains_key("lon") {
                    let lat = obj.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let lon = obj.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    Ok(PayloadFieldValue::GeoPoint { lat: OrderedFloat(lat), lon: OrderedFloat(lon) })
                } else {
                    Err(DbError::InvalidOperation("Unsupported object format".to_string()))
                }
            }
            JsonValue::Null => Err(DbError::InvalidOperation("Null values not supported".to_string())),
        }
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self::new()
    }
}

/// An index for payload fields to enable fast filtering.
pub struct PayloadIndex {
    /// Index for string fields.
    string_index: HashMap<String, HashMap<String, HashSet<EmbeddingId>>>,
    
    /// Index for integer fields.
    integer_index: HashMap<String, BTreeMap<i64, HashSet<EmbeddingId>>>,
    
    /// Index for float fields.
    float_index: HashMap<String, BTreeMap<OrderedFloat<f64>, HashSet<EmbeddingId>>>,
    
    /// Index for boolean fields.
    boolean_index: HashMap<String, HashMap<bool, HashSet<EmbeddingId>>>,
    
    /// Index for geo fields.
    geo_index: HashMap<String, HashMap<EmbeddingId, GeoPoint>>,
    
    /// Map from embedding ID to payload.
    id_to_payload: HashMap<EmbeddingId, Payload>,
}

impl PayloadIndex {
    /// Creates a new payload index.
    pub fn new() -> Self {
        Self {
            string_index: HashMap::new(),
            integer_index: HashMap::new(),
            float_index: HashMap::new(),
            boolean_index: HashMap::new(),
            geo_index: HashMap::new(),
            id_to_payload: HashMap::new(),
        }
    }
    
    /// Adds a payload to the index.
    pub fn add_payload(&mut self, id: EmbeddingId, payload: Payload) -> DbResult<()> {
        // Store the payload
        self.id_to_payload.insert(id.clone(), payload.clone());
        
        // Index each field
        for (field_name, field_value) in payload.fields {
            match field_value {
                PayloadFieldValue::String(s) => {
                    self.string_index
                        .entry(field_name)
                        .or_insert_with(HashMap::new)
                        .entry(s)
                        .or_insert_with(HashSet::new)
                        .insert(id.clone());
                }
                PayloadFieldValue::Integer(i) => {
                    self.integer_index
                        .entry(field_name)
                        .or_insert_with(BTreeMap::new)
                        .entry(i)
                        .or_insert_with(HashSet::new)
                        .insert(id.clone());
                }
                PayloadFieldValue::Float(f) => {
                    self.float_index
                        .entry(field_name)
                        .or_insert_with(BTreeMap::new)
                        .entry(f)
                        .or_insert_with(HashSet::new)
                        .insert(id.clone());
                }
                PayloadFieldValue::Boolean(b) => {
                    self.boolean_index
                        .entry(field_name)
                        .or_insert_with(HashMap::new)
                        .entry(b)
                        .or_insert_with(HashSet::new)
                        .insert(id.clone());
                }
                PayloadFieldValue::GeoPoint { lat, lon } => {
                    self.geo_index
                        .entry(field_name)
                        .or_insert_with(HashMap::new)
                        .insert(id.clone(), GeoPoint { lat, lon });
                }
                PayloadFieldValue::List(_) => {
                    // Lists are not indexed in this simple implementation
                    // In a real implementation, we would index each element
                }
            }
        }
        
        Ok(())
    }
    
    /// Removes a payload from the index.
    pub fn remove_payload(&mut self, id: &EmbeddingId) -> DbResult<bool> {
        if let Some(payload) = self.id_to_payload.remove(id) {
            // Remove from field indexes
            for (field_name, field_value) in payload.fields {
                match field_value {
                    PayloadFieldValue::String(s) => {
                        if let Some(field_index) = self.string_index.get_mut(&field_name) {
                            if let Some(value_index) = field_index.get_mut(&s) {
                                value_index.remove(id);
                                if value_index.is_empty() {
                                    field_index.remove(&s);
                                }
                            }
                            if field_index.is_empty() {
                                self.string_index.remove(&field_name);
                            }
                        }
                    }
                    PayloadFieldValue::Integer(i) => {
                        if let Some(field_index) = self.integer_index.get_mut(&field_name) {
                            if let Some(value_index) = field_index.get_mut(&i) {
                                value_index.remove(id);
                                if value_index.is_empty() {
                                    field_index.remove(&i);
                                }
                            }
                            if field_index.is_empty() {
                                self.integer_index.remove(&field_name);
                            }
                        }
                    }
                    PayloadFieldValue::Float(f) => {
                        if let Some(field_index) = self.float_index.get_mut(&field_name) {
                            if let Some(value_index) = field_index.get_mut(&f) {
                                value_index.remove(id);
                                if value_index.is_empty() {
                                    field_index.remove(&f);
                                }
                            }
                            if field_index.is_empty() {
                                self.float_index.remove(&field_name);
                            }
                        }
                    }
                    PayloadFieldValue::Boolean(b) => {
                        if let Some(field_index) = self.boolean_index.get_mut(&field_name) {
                            if let Some(value_index) = field_index.get_mut(&b) {
                                value_index.remove(id);
                                if value_index.is_empty() {
                                    field_index.remove(&b);
                                }
                            }
                            if field_index.is_empty() {
                                self.boolean_index.remove(&field_name);
                            }
                        }
                    }
                    PayloadFieldValue::GeoPoint { lat, lon } => {
                        if let Some(field_index) = self.geo_index.get_mut(&field_name) {
                            field_index.remove(id);
                            if field_index.is_empty() {
                                self.geo_index.remove(&field_name);
                            }
                        }
                    }
                    PayloadFieldValue::List(_) => {
                        // Lists are not indexed in this simple implementation
                    }
                }
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Gets the payload for an embedding ID.
    pub fn get_payload(&self, id: &EmbeddingId) -> Option<&Payload> {
        self.id_to_payload.get(id)
    }
    
    /// Filters embedding IDs based on a payload filter.
    pub fn filter(&self, filter: &PayloadFilter) -> HashSet<EmbeddingId> {
        let mut result = HashSet::new();
        
        // Start with all IDs if there are no must conditions
        if filter.must.is_empty() {
            result.extend(self.id_to_payload.keys().cloned());
        }
        
        // Apply must conditions
        for condition in &filter.must {
            let ids = self.get_ids_for_condition(condition);
            if result.is_empty() {
                result.extend(ids);
            } else {
                result.retain(|id| ids.contains(id));
            }
            // Early exit if no results
            if result.is_empty() {
                break;
            }
        }
        
        // Apply must_not conditions
        for condition in &filter.must_not {
            let ids = self.get_ids_for_condition(condition);
            result.retain(|id| !ids.contains(id));
            // Early exit if no results
            if result.is_empty() {
                break;
            }
        }
        
        // Apply should conditions
        if !filter.should.is_empty() {
            let mut should_ids = HashSet::new();
            for condition in &filter.should {
                should_ids.extend(self.get_ids_for_condition(condition));
            }
            result.retain(|id| should_ids.contains(id));
        }
        
        result
    }
    
    /// Gets embedding IDs that match a condition.
    fn get_ids_for_condition(&self, condition: &PayloadCondition) -> HashSet<EmbeddingId> {
        let mut result = HashSet::new();
        
        // This is a simplified implementation
        // In a real implementation, we would need to specify which field to check
        match condition {
            PayloadCondition::Match { value } => {
                match value {
                    PayloadFieldValue::String(s) => {
                        for field_index in self.string_index.values() {
                            if let Some(ids) = field_index.get(s) {
                                result.extend(ids);
                            }
                        }
                    }
                    PayloadFieldValue::Integer(i) => {
                        for field_index in self.integer_index.values() {
                            if let Some(ids) = field_index.get(i) {
                                result.extend(ids);
                            }
                        }
                    }
                    PayloadFieldValue::Float(f) => {
                        for field_index in self.float_index.values() {
                            // This is approximate since f64 keys in BTreeMap are compared exactly
                            if let Some(ids) = field_index.get(f) {
                                result.extend(ids);
                            }
                        }
                    }
                    PayloadFieldValue::Boolean(b) => {
                        for field_index in self.boolean_index.values() {
                            if let Some(ids) = field_index.get(b) {
                                result.extend(ids);
                            }
                        }
                    }
                    PayloadFieldValue::GeoPoint { lat, lon } => {
                        // For geo points, we would need to do a radius search
                        // This is a simplified implementation
                        for field_index in self.geo_index.values() {
                            for (id, point) in field_index {
                                let distance = haversine_distance(lat.into_inner(), lon.into_inner(), point.lat.into_inner(), point.lon.into_inner());
                                if distance <= 1000.0 { // 1km radius for example
                                    result.insert(id);
                                }
                            }
                        }
                    }
                    PayloadFieldValue::List(_) => {
                        // Lists are not indexed in this simple implementation
                    }
                }
            }
            PayloadCondition::Range { from, to } => {
                // Check integer and float indexes
                for field_index in self.integer_index.values() {
                    let range = match (from, to) {
                        (Some(f), Some(t)) => {
                            field_index.range((std::ops::Bound::Included(&(f.round() as i64)), std::ops::Bound::Included(&(t.round() as i64))))
                        }
                        (Some(f), None) => {
                            field_index.range((std::ops::Bound::Included(f.round() as i64), std::ops::Bound::Unbounded))
                        }
                        (None, Some(t)) => {
                            field_index.range((std::ops::Bound::Unbounded, std::ops::Bound::Included(t.round() as i64)))
                        }
                        (None, None) => {
                            field_index.range(..)
                        }
                    };
                    
                    for (_, ids) in range {
                        result.extend(ids);
                    }
                }
                
                for field_index in self.float_index.values() {
                    let range = match (from, to) {
                        (Some(f), Some(t)) => {
                            field_index.range((std::ops::Bound::Included(OrderedFloat(f.into_inner())), std::ops::Bound::Included(OrderedFloat(t.into_inner()))))
                        }
                        (Some(f), None) => {
                            field_index.range((std::ops::Bound::Included(OrderedFloat(f.into_inner())), std::ops::Bound::Unbounded))
                        }
                        (None, Some(t)) => {
                            field_index.range((std::ops::Bound::Unbounded, std::ops::Bound::Included(OrderedFloat(t.into_inner()))))
                        }
                        (None, None) => {
                            field_index.range(..)
                        }
                    };
                    
                    for (_, ids) in range {
                        result.extend(ids);
                    }
                }
            }
            PayloadCondition::GeoRadius { center, radius } => {
                // Check geo indexes
                for field_index in self.geo_index.values() {
                    for (id, point) in field_index {
                        let distance = haversine_distance(center.lat.into_inner(), center.lon.into_inner(), point.lat.into_inner(), point.lon.into_inner());
                        if distance <= radius.into_inner() {
                            result.insert(id);
                        }
                    }
                }
            }
        }
        
        result.into_iter().cloned().collect()
    }
}

impl Default for PayloadIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// A hybrid vector index that combines vector search with payload filtering.
pub struct HybridVectorIndex {
    /// The underlying vector index
    vector_index: super::vector::VectorIndex,
    
    /// The payload index
    payload_index: PayloadIndex,
}

impl HybridVectorIndex {
    /// Creates a new hybrid vector index.
    pub fn new(persist_path: impl AsRef<std::path::Path>) -> DbResult<Self> {
        Ok(Self {
            vector_index: super::vector::VectorIndex::new(persist_path)?,
            payload_index: PayloadIndex::new(),
        })
    }
    
    /// Adds a vector with payload to the index.
    pub fn add_vector_with_payload(
        &mut self,
        id: &str,
        vector: Vec<f32>,
        payload: Payload,
    ) -> DbResult<()> {
        // Add to vector index
        self.vector_index.add_vector(id, vector)?;
        
        // Add to payload index
        self.payload_index.add_payload(EmbeddingId::from(id), payload)?;
        
        Ok(())
    }
    
    /// Removes a vector from the index.
    pub fn remove_vector(&mut self, id: &str) -> DbResult<bool> {
        // Remove from vector index
        let vector_removed = self.vector_index.remove_vector(id)?;
        
        // Remove from payload index
        let payload_removed = self.payload_index.remove_payload(&EmbeddingId::from(id))?;
        
        Ok(vector_removed || payload_removed)
    }
    
    /// Searches for the k nearest neighbors of a query vector with payload filtering.
    pub fn search_with_filter(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&PayloadFilter>,
    ) -> DbResult<Vec<super::vector::SearchResult>> {
        // First, get candidates from payload filter if provided
        let candidates = if let Some(filter) = filter {
            Some(self.payload_index.filter(filter))
        } else {
            None
        };
        
        // Perform vector search
        let mut results = self.vector_index.search(query, k)?;
        
        // Filter results by payload if needed
        if let Some(candidates) = candidates {
            results.retain(|result| candidates.contains(&result.id));
        }
        
        Ok(results)
    }
    
    /// Gets the payload for an embedding ID.
    pub fn get_payload(&self, id: &EmbeddingId) -> Option<&Payload> {
        self.payload_index.get_payload(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_payload_creation() {
        let mut payload = Payload::new();
        payload.add_field("name".to_string(), PayloadFieldValue::String("test".to_string()));
        payload.add_field("age".to_string(), PayloadFieldValue::Integer(25));
        payload.add_field("active".to_string(), PayloadFieldValue::Boolean(true));
        
        assert_eq!(payload.get_field("name"), Some(&PayloadFieldValue::String("test".to_string())));
        assert_eq!(payload.get_field("age"), Some(&PayloadFieldValue::Integer(25)));
        assert_eq!(payload.get_field("active"), Some(&PayloadFieldValue::Boolean(true)));
    }
    
    #[test]
    fn test_payload_index() {
        let mut index = PayloadIndex::new();
        
        let id1 = EmbeddingId::from("vector1");
        let mut payload1 = Payload::new();
        payload1.add_field("category".to_string(), PayloadFieldValue::String("A".to_string()));
        payload1.add_field("score".to_string(), PayloadFieldValue::Float(OrderedFloat(0.8)));
        
        let id2 = EmbeddingId::from("vector2");
        let mut payload2 = Payload::new();
        payload2.add_field("category".to_string(), PayloadFieldValue::String("B".to_string()));
        payload2.add_field("score".to_string(), PayloadFieldValue::Float(OrderedFloat(0.9)));
        
        index.add_payload(id1.clone(), payload1).unwrap();
        index.add_payload(id2.clone(), payload2).unwrap();
        
        // Test filtering
        let mut filter = PayloadFilter::new();
        filter = filter.must(PayloadCondition::Match {
            value: PayloadFieldValue::String("A".to_string()),
        });
        
        let results = index.filter(&filter);
        assert!(results.contains(&id1));
        assert!(!results.contains(&id2));
    }
    
    #[test]
    fn test_hybrid_vector_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        
        let mut index = HybridVectorIndex::new(&index_path).unwrap();
        
        let mut payload = Payload::new();
        payload.add_field("category".to_string(), PayloadFieldValue::String("test".to_string()));
        
        index.add_vector_with_payload("vector1", vec![1.0, 0.0, 0.0], payload).unwrap();
        
        let results = index.search_with_filter(&[1.0, 0.0, 0.0], 1, None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, EmbeddingId::from("vector1"));
    }
}