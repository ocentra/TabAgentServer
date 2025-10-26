//! Advanced distance metrics for vector similarity search.
//!
//! This module provides implementations of various distance metrics
//! commonly used in vector similarity search, including Euclidean,
//! Manhattan, Dot Product, Jaccard, and Hamming distances. These implementations
//! follow the Rust Architecture Guidelines for safety, performance, and clarity.
//!
//! For better performance, SIMD-optimized versions of these metrics are available
//! in the `simd_distance_metrics` module.

use common::DbResult;

/// A trait for distance metrics.
pub trait DistanceMetric: Send + Sync {
    /// Calculates the distance between two vectors.
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32>;
    
    /// Calculates the similarity between two vectors.
    /// 
    /// For metrics where lower distance means higher similarity,
    /// this function should convert distance to similarity.
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let distance = self.distance(a, b)?;
        Ok(1.0 / (1.0 + distance))
    }
    
    /// Gets the name of this distance metric.
    fn name(&self) -> &'static str;
}

/// Cosine distance metric.
///
/// Cosine distance = 1 - cosine similarity
/// Cosine similarity = (A · B) / (||A|| × ||B||)
pub struct CosineMetric;

impl CosineMetric {
    /// Creates a new cosine distance metric.
    pub fn new() -> Self {
        Self
    }
    
    /// Calculates cosine similarity between two vectors.
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(1.0);
        }
        
        // Calculate dot product
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        
        // Calculate magnitudes
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        // Handle zero vectors
        if magnitude_a.abs() < f32::EPSILON || magnitude_b.abs() < f32::EPSILON {
            return Ok(0.0);
        }
        
        let similarity = dot_product / (magnitude_a * magnitude_b);
        
        // Clamp to [-1, 1] to handle floating point errors
        Ok(similarity.clamp(-1.0, 1.0))
    }
}

impl DistanceMetric for CosineMetric {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let similarity = Self::cosine_similarity(a, b)?;
        // Cosine distance = 1 - cosine similarity
        Ok(1.0 - similarity)
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        Self::cosine_similarity(a, b)
    }
    
    fn name(&self) -> &'static str {
        "Cosine"
    }
}

impl Default for CosineMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// Euclidean distance metric.
///
/// Euclidean distance = √(Σ(Ai - Bi)²)
pub struct EuclideanMetric;

impl EuclideanMetric {
    /// Creates a new Euclidean distance metric.
    pub fn new() -> Self {
        Self
    }
}

impl DistanceMetric for EuclideanMetric {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        let sum_squares: f32 = a.iter().zip(b.iter()).map(|(x, y)| {
            let diff = x - y;
            diff * diff
        }).sum();
        
        Ok(sum_squares.sqrt())
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let distance = self.distance(a, b)?;
        // Convert distance to similarity (higher distance = lower similarity)
        // Using exponential decay: similarity = e^(-distance)
        Ok((-distance).exp())
    }
    
    fn name(&self) -> &'static str {
        "Euclidean"
    }
}

impl Default for EuclideanMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// Manhattan distance metric (L1 distance).
///
/// Manhattan distance = Σ|Ai - Bi|
pub struct ManhattanMetric;

impl ManhattanMetric {
    /// Creates a new Manhattan distance metric.
    pub fn new() -> Self {
        Self
    }
}

impl DistanceMetric for ManhattanMetric {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        let sum_abs_diff: f32 = a.iter().zip(b.iter()).map(|(x, y)| {
            (x - y).abs()
        }).sum();
        
        Ok(sum_abs_diff)
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let distance = self.distance(a, b)?;
        // Convert distance to similarity (higher distance = lower similarity)
        // Using inverse: similarity = 1 / (1 + distance)
        Ok(1.0 / (1.0 + distance))
    }
    
    fn name(&self) -> &'static str {
        "Manhattan"
    }
}

impl Default for ManhattanMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// Dot product distance metric.
///
/// Dot product distance = -A · B (negative because we want to minimize distance
/// but maximize dot product for similarity)
pub struct DotProductMetric;

impl DotProductMetric {
    /// Creates a new dot product distance metric.
    pub fn new() -> Self {
        Self
    }
}

impl DistanceMetric for DotProductMetric {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        // Dot product distance = -dot_product
        // (negative because higher dot product = higher similarity)
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        Ok(-dot_product)
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        // Dot product similarity = dot_product
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        Ok(dot_product)
    }
    
    fn name(&self) -> &'static str {
        "DotProduct"
    }
}

impl Default for DotProductMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// Jaccard distance metric for binary vectors.
///
/// Jaccard distance = 1 - Jaccard similarity
/// Jaccard similarity = |A ∩ B| / |A ∪ B|
pub struct JaccardMetric {
    /// Threshold for converting continuous values to binary
    threshold: f32,
}

impl JaccardMetric {
    /// Creates a new Jaccard distance metric with default threshold (0.5).
    pub fn new() -> Self {
        Self { threshold: 0.5 }
    }
    
    /// Creates a new Jaccard distance metric with a custom threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - The threshold value for converting continuous values to binary.
    ///   Values >= threshold are considered true, values < threshold are considered false.
    pub fn with_threshold(threshold: f32) -> Self {
        Self { threshold }
    }
    
    /// Converts f32 vectors to binary vectors using the metric's threshold.
    pub fn to_binary(&self, vector: &[f32]) -> Vec<bool> {
        vector.iter().map(|&x| x >= self.threshold).collect()
    }
    
    /// Calculates Jaccard similarity between two binary vectors.
    ///
    /// # Arguments
    ///
    /// * `a` - First binary vector
    /// * `b` - Second binary vector
    ///
    /// # Returns
    ///
    /// Jaccard similarity value between 0.0 and 1.0
    pub fn jaccard_similarity_binary(a: &[bool], b: &[bool]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(1.0);
        }
        
        let mut intersection = 0;
        let mut union = 0;
        
        for (a_bit, b_bit) in a.iter().zip(b.iter()) {
            if *a_bit || *b_bit {
                union += 1;
                if *a_bit && *b_bit {
                    intersection += 1;
                }
            }
        }
        
        if union == 0 {
            return Ok(1.0);
        }
        
        Ok(intersection as f32 / union as f32)
    }
}

impl DistanceMetric for JaccardMetric {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        // Convert to binary vectors using the metric's threshold
        let a_binary = self.to_binary(a);
        let b_binary = self.to_binary(b);
        
        let similarity = Self::jaccard_similarity_binary(&a_binary, &b_binary)?;
        // Jaccard distance = 1 - Jaccard similarity
        Ok(1.0 - similarity)
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(1.0);
        }
        
        // Convert to binary vectors using the metric's threshold
        let a_binary = self.to_binary(a);
        let b_binary = self.to_binary(b);
        
        Self::jaccard_similarity_binary(&a_binary, &b_binary)
    }
    
    fn name(&self) -> &'static str {
        "Jaccard"
    }
}

impl Default for JaccardMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// Hamming distance metric for binary vectors.
///
/// Hamming distance = number of positions where bits differ
pub struct HammingMetric {
    /// Threshold for converting continuous values to binary
    threshold: f32,
}

impl HammingMetric {
    /// Creates a new Hamming distance metric with default threshold (0.5).
    pub fn new() -> Self {
        Self { threshold: 0.5 }
    }
    
    /// Creates a new Hamming distance metric with a custom threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - The threshold value for converting continuous values to binary.
    ///   Values >= threshold are considered true, values < threshold are considered false.
    pub fn with_threshold(threshold: f32) -> Self {
        Self { threshold }
    }
    
    /// Converts f32 vectors to binary vectors using the metric's threshold.
    pub fn to_binary(&self, vector: &[f32]) -> Vec<bool> {
        vector.iter().map(|&x| x >= self.threshold).collect()
    }
    
    /// Calculates Hamming distance between two binary vectors.
    ///
    /// # Arguments
    ///
    /// * `a` - First binary vector
    /// * `b` - Second binary vector
    ///
    /// # Returns
    ///
    /// Hamming distance (number of positions where bits differ)
    pub fn hamming_distance_binary(a: &[bool], b: &[bool]) -> DbResult<usize> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        Ok(a.iter().zip(b.iter())
            .filter(|(a_bit, b_bit)| a_bit != b_bit)
            .count())
    }
}

impl DistanceMetric for HammingMetric {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        // Convert to binary vectors using the metric's threshold
        let a_binary = self.to_binary(a);
        let b_binary = self.to_binary(b);
        
        let hamming_distance = Self::hamming_distance_binary(&a_binary, &b_binary)?;
        Ok(hamming_distance as f32)
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let distance = self.distance(a, b)?;
        let max_distance = a.len() as f32;
        
        if max_distance == 0.0 {
            return Ok(1.0);
        }
        
        // Convert distance to similarity: similarity = 1 - (distance / max_distance)
        Ok(1.0 - (distance / max_distance))
    }
    
    fn name(&self) -> &'static str {
        "Hamming"
    }
}

impl Default for HammingMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// Enum for different distance metrics.
pub enum DistanceMetricType {
    /// Cosine distance
    Cosine(CosineMetric),
    
    /// Euclidean distance
    Euclidean(EuclideanMetric),
    
    /// Manhattan distance
    Manhattan(ManhattanMetric),
    
    /// Dot product distance
    DotProduct(DotProductMetric),
    
    /// Jaccard distance
    Jaccard(JaccardMetric),
    
    /// Hamming distance
    Hamming(HammingMetric),
}

impl DistanceMetric for DistanceMetricType {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        match self {
            DistanceMetricType::Cosine(metric) => metric.distance(a, b),
            DistanceMetricType::Euclidean(metric) => metric.distance(a, b),
            DistanceMetricType::Manhattan(metric) => metric.distance(a, b),
            DistanceMetricType::DotProduct(metric) => metric.distance(a, b),
            DistanceMetricType::Jaccard(metric) => metric.distance(a, b),
            DistanceMetricType::Hamming(metric) => metric.distance(a, b),
        }
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        match self {
            DistanceMetricType::Cosine(metric) => metric.similarity(a, b),
            DistanceMetricType::Euclidean(metric) => metric.similarity(a, b),
            DistanceMetricType::Manhattan(metric) => metric.similarity(a, b),
            DistanceMetricType::DotProduct(metric) => metric.similarity(a, b),
            DistanceMetricType::Jaccard(metric) => metric.similarity(a, b),
            DistanceMetricType::Hamming(metric) => metric.similarity(a, b),
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            DistanceMetricType::Cosine(metric) => metric.name(),
            DistanceMetricType::Euclidean(metric) => metric.name(),
            DistanceMetricType::Manhattan(metric) => metric.name(),
            DistanceMetricType::DotProduct(metric) => metric.name(),
            DistanceMetricType::Jaccard(metric) => metric.name(),
            DistanceMetricType::Hamming(metric) => metric.name(),
        }
    }
}

/// A distance metric that can be selected at runtime.
pub struct DynamicDistanceMetric {
    metric: DistanceMetricType,
}

impl DynamicDistanceMetric {
    /// Creates a new dynamic distance metric.
    pub fn new(metric_type: DistanceMetricType) -> Self {
        Self { metric: metric_type }
    }
    
    /// Creates a new dynamic distance metric from a string name.
    pub fn from_name(name: &str) -> DbResult<Self> {
        let metric = match name.to_lowercase().as_str() {
            "cosine" => DistanceMetricType::Cosine(CosineMetric::new()),
            "euclidean" => DistanceMetricType::Euclidean(EuclideanMetric::new()),
            "manhattan" => DistanceMetricType::Manhattan(ManhattanMetric::new()),
            "dotproduct" => DistanceMetricType::DotProduct(DotProductMetric::new()),
            "jaccard" => DistanceMetricType::Jaccard(JaccardMetric::new()),
            "hamming" => DistanceMetricType::Hamming(HammingMetric::new()),
            _ => return Err(common::DbError::InvalidOperation(
                format!("Unknown distance metric: {}", name)
            )),
        };
        
        Ok(Self { metric })
    }
    
    /// Creates a new dynamic distance metric from a string name with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the distance metric
    /// * `threshold` - Optional threshold parameter for Jaccard and Hamming metrics
    pub fn from_name_with_params(name: &str, threshold: Option<f32>) -> DbResult<Self> {
        let metric = match name.to_lowercase().as_str() {
            "cosine" => DistanceMetricType::Cosine(CosineMetric::new()),
            "euclidean" => DistanceMetricType::Euclidean(EuclideanMetric::new()),
            "manhattan" => DistanceMetricType::Manhattan(ManhattanMetric::new()),
            "dotproduct" => DistanceMetricType::DotProduct(DotProductMetric::new()),
            "jaccard" => {
                let jaccard = if let Some(t) = threshold {
                    JaccardMetric::with_threshold(t)
                } else {
                    JaccardMetric::new()
                };
                DistanceMetricType::Jaccard(jaccard)
            },
            "hamming" => {
                let hamming = if let Some(t) = threshold {
                    HammingMetric::with_threshold(t)
                } else {
                    HammingMetric::new()
                };
                DistanceMetricType::Hamming(hamming)
            },
            _ => return Err(common::DbError::InvalidOperation(
                format!("Unknown distance metric: {}", name)
            )),
        };
        
        Ok(Self { metric })
    }
}

impl DistanceMetric for DynamicDistanceMetric {
    fn distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        self.metric.distance(a, b)
    }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        self.metric.similarity(a, b)
    }
    
    fn name(&self) -> &'static str {
        self.metric.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_metric() {
        let metric = CosineMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
        
        // Test orthogonal vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 1.0).abs() < f32::EPSILON);
        assert!((similarity - 0.0).abs() < f32::EPSILON);
        
        // Test opposite vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 2.0).abs() < f32::EPSILON);
        assert!((similarity - (-1.0)).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_euclidean_metric() {
        let metric = EuclideanMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        
        // Test unit distance
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 1.0).abs() < f32::EPSILON);
        
        // Test 2D diagonal
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 1.0];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 2.0f32.sqrt()).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_manhattan_metric() {
        let metric = ManhattanMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        
        // Test unit distance
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 1.0).abs() < f32::EPSILON);
        
        // Test 2D Manhattan distance
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 1.0];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 2.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_dot_product_metric() {
        let metric = DotProductMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - (-1.0)).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
        
        // Test orthogonal vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        assert!((similarity - 0.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_jaccard_metric() {
        // Test with default threshold
        let metric = JaccardMetric::new();
        
        // Test identical binary vectors
        let a = vec![1.0, 0.0, 1.0, 0.0];
        let b = vec![1.0, 0.0, 1.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
        
        // Test completely different binary vectors
        let a = vec![1.0, 1.0, 1.0, 1.0];
        let b = vec![0.0, 0.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 1.0).abs() < f32::EPSILON);
        assert!((similarity - 0.0).abs() < f32::EPSILON);
        
        // Test with custom threshold
        let metric = JaccardMetric::with_threshold(0.7);
        let a = vec![0.8, 0.6, 0.9, 0.4];
        let b = vec![0.7, 0.5, 0.8, 0.3];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        // With threshold 0.7, both vectors become [1, 0, 1, 0]
        assert!((distance - 0.0).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_hamming_metric() {
        // Test with default threshold
        let metric = HammingMetric::new();
        
        // Test identical binary vectors
        let a = vec![1.0, 0.0, 1.0, 0.0];
        let b = vec![1.0, 0.0, 1.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
        
        // Test completely different binary vectors
        let a = vec![1.0, 1.0, 1.0, 1.0];
        let b = vec![0.0, 0.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        assert!((distance - 4.0).abs() < f32::EPSILON);
        assert!((similarity - 0.0).abs() < f32::EPSILON);
        
        // Test with custom threshold
        let metric = HammingMetric::with_threshold(0.7);
        let a = vec![0.8, 0.6, 0.9, 0.4];
        let b = vec![0.7, 0.5, 0.8, 0.3];
        let distance = metric.distance(&a, &b).unwrap();
        let similarity = metric.similarity(&a, &b).unwrap();
        // With threshold 0.7, both vectors become [1, 0, 1, 0]
        assert!((distance - 0.0).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_dynamic_metric() {
        let metric = DynamicDistanceMetric::from_name("cosine").unwrap();
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        
        let metric = DynamicDistanceMetric::from_name("euclidean").unwrap();
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        
        // Test with custom threshold
        let metric = DynamicDistanceMetric::from_name_with_params("jaccard", Some(0.7)).unwrap();
        let a = vec![0.8, 0.6, 0.9, 0.4];
        let b = vec![0.7, 0.5, 0.8, 0.3];
        let distance = metric.distance(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_binary_helpers() {
        let jaccard = JaccardMetric::with_threshold(0.5);
        let hamming = HammingMetric::with_threshold(0.5);
        
        let vec = vec![0.2, 0.6, 0.8, 0.4];
        let expected = vec![false, true, true, false];
        
        assert_eq!(jaccard.to_binary(&vec), expected);
        assert_eq!(hamming.to_binary(&vec), expected);
    }
}