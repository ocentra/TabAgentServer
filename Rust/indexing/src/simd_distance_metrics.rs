//! SIMD-optimized distance metrics for vector similarity search.
//!
//! This module provides SIMD-optimized implementations of various distance metrics
//! commonly used in vector similarity search. These implementations leverage
//! the wide crate for SIMD operations to achieve better performance on modern CPUs.
//! The implementations follow the Rust Architecture Guidelines for safety,
//! performance, and clarity.

use common::DbResult;
use wide::f32x4;

/// A trait for SIMD-optimized distance metrics.
pub trait SimdDistanceMetric: Send + Sync {
    /// Calculates the distance between two vectors using SIMD optimization.
    fn simd_distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32>;
    
    /// Calculates the similarity between two vectors using SIMD optimization.
    /// 
    /// For metrics where lower distance means higher similarity,
    /// this function should convert distance to similarity.
    fn simd_similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let distance = self.simd_distance(a, b)?;
        Ok(1.0 / (1.0 + distance))
    }
    
    /// Gets the name of this distance metric.
    fn name(&self) -> &'static str;
}

/// SIMD-optimized cosine distance metric.
///
/// Cosine distance = 1 - cosine similarity
/// Cosine similarity = (A · B) / (||A|| × ||B||)
pub struct SimdCosineMetric;

impl SimdCosineMetric {
    /// Creates a new SIMD-optimized cosine distance metric.
    pub fn new() -> Self {
        Self
    }
    
    /// Calculates cosine similarity between two vectors using SIMD optimization.
    pub fn simd_cosine_similarity(a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(1.0);
        }
        
        // Calculate dot product using SIMD
        let dot_product = Self::simd_dot_product(a, b);
        
        // Calculate magnitudes using SIMD
        let magnitude_a = Self::simd_magnitude(a);
        let magnitude_b = Self::simd_magnitude(b);
        
        // Handle zero vectors
        if magnitude_a.abs() < f32::EPSILON || magnitude_b.abs() < f32::EPSILON {
            return Ok(0.0);
        }
        
        let similarity = dot_product / (magnitude_a * magnitude_b);
        
        // Clamp to [-1, 1] to handle floating point errors
        Ok(similarity.clamp(-1.0, 1.0))
    }
    
    /// Calculates the dot product of two vectors using SIMD optimization.
    fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
        let mut sum = f32x4::ZERO;
        let mut i = 0;
        
        // Process 4 elements at a time using SIMD
        while i + 4 <= a.len() {
            let a_vec = f32x4::from([
                a[i],
                a[i + 1],
                a[i + 2],
                a[i + 3],
            ]);
            
            let b_vec = f32x4::from([
                b[i],
                b[i + 1],
                b[i + 2],
                b[i + 3],
            ]);
            
            sum += a_vec * b_vec;
            i += 4;
        }
        
        // Sum the SIMD results
        let sum_array = sum.to_array();
        let mut result = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements (less than 4)
        while i < a.len() {
            result += a[i] * b[i];
            i += 1;
        }
        
        result
    }
    
    /// Calculates the magnitude of a vector using SIMD optimization.
    fn simd_magnitude(vector: &[f32]) -> f32 {
        let mut sum = f32x4::ZERO;
        let mut i = 0;
        
        // Process 4 elements at a time using SIMD
        while i + 4 <= vector.len() {
            let vec = f32x4::from([
                vector[i],
                vector[i + 1],
                vector[i + 2],
                vector[i + 3],
            ]);
            
            sum += vec * vec;
            i += 4;
        }
        
        // Sum the SIMD results
        let sum_array = sum.to_array();
        let mut result = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements (less than 4)
        while i < vector.len() {
            result += vector[i] * vector[i];
            i += 1;
        }
        
        result.sqrt()
    }
}

impl SimdDistanceMetric for SimdCosineMetric {
    fn simd_distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let similarity = Self::simd_cosine_similarity(a, b)?;
        // Cosine distance = 1 - cosine similarity
        Ok(1.0 - similarity)
    }
    
    fn simd_similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        Self::simd_cosine_similarity(a, b)
    }
    
    fn name(&self) -> &'static str {
        "SIMD-Cosine"
    }
}

impl Default for SimdCosineMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// SIMD-optimized Euclidean distance metric.
///
/// Euclidean distance = √(Σ(Ai - Bi)²)
pub struct SimdEuclideanMetric;

impl SimdEuclideanMetric {
    /// Creates a new SIMD-optimized Euclidean distance metric.
    pub fn new() -> Self {
        Self
    }
    
    /// Calculates the squared Euclidean distance between two vectors using SIMD optimization.
    fn simd_squared_euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        let mut sum = f32x4::ZERO;
        let mut i = 0;
        
        // Process 4 elements at a time using SIMD
        while i + 4 <= a.len() {
            let a_vec = f32x4::from([
                a[i],
                a[i + 1],
                a[i + 2],
                a[i + 3],
            ]);
            
            let b_vec = f32x4::from([
                b[i],
                b[i + 1],
                b[i + 2],
                b[i + 3],
            ]);
            
            let diff = a_vec - b_vec;
            sum += diff * diff;
            i += 4;
        }
        
        // Sum the SIMD results
        let sum_array = sum.to_array();
        let mut result = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements (less than 4)
        while i < a.len() {
            let diff = a[i] - b[i];
            result += diff * diff;
            i += 1;
        }
        
        result
    }
}

impl SimdDistanceMetric for SimdEuclideanMetric {
    fn simd_distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        let squared_distance = Self::simd_squared_euclidean_distance(a, b);
        Ok(squared_distance.sqrt())
    }
    
    fn simd_similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let distance = self.simd_distance(a, b)?;
        // Convert distance to similarity (higher distance = lower similarity)
        // Using exponential decay: similarity = e^(-distance)
        Ok((-distance).exp())
    }
    
    fn name(&self) -> &'static str {
        "SIMD-Euclidean"
    }
}

impl Default for SimdEuclideanMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// SIMD-optimized Manhattan distance metric (L1 distance).
///
/// Manhattan distance = Σ|Ai - Bi|
pub struct SimdManhattanMetric;

impl SimdManhattanMetric {
    /// Creates a new SIMD-optimized Manhattan distance metric.
    pub fn new() -> Self {
        Self
    }
    
    /// Calculates the Manhattan distance between two vectors using SIMD optimization.
    fn simd_manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
        let mut sum = f32x4::ZERO;
        let mut i = 0;
        
        // Process 4 elements at a time using SIMD
        while i + 4 <= a.len() {
            let a_vec = f32x4::from([
                a[i],
                a[i + 1],
                a[i + 2],
                a[i + 3],
            ]);
            
            let b_vec = f32x4::from([
                b[i],
                b[i + 1],
                b[i + 2],
                b[i + 3],
            ]);
            
            let diff = a_vec - b_vec;
            sum += diff.abs();
            i += 4;
        }
        
        // Sum the SIMD results
        let sum_array = sum.to_array();
        let mut result = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements (less than 4)
        while i < a.len() {
            result += (a[i] - b[i]).abs();
            i += 1;
        }
        
        result
    }
}

impl SimdDistanceMetric for SimdManhattanMetric {
    fn simd_distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        Ok(Self::simd_manhattan_distance(a, b))
    }
    
    fn simd_similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        let distance = self.simd_distance(a, b)?;
        // Convert distance to similarity (higher distance = lower similarity)
        // Using inverse: similarity = 1 / (1 + distance)
        Ok(1.0 / (1.0 + distance))
    }
    
    fn name(&self) -> &'static str {
        "SIMD-Manhattan"
    }
}

impl Default for SimdManhattanMetric {
    fn default() -> Self {
        Self::new()
    }
}

/// SIMD-optimized dot product distance metric.
///
/// Dot product distance = -A · B (negative because we want to minimize distance
/// but maximize dot product for similarity)
pub struct SimdDotProductMetric;

impl SimdDotProductMetric {
    /// Creates a new SIMD-optimized dot product distance metric.
    pub fn new() -> Self {
        Self
    }
}

impl SimdDistanceMetric for SimdDotProductMetric {
    fn simd_distance(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        // Calculate dot product using SIMD
        let mut sum = f32x4::ZERO;
        let mut i = 0;
        
        // Process 4 elements at a time using SIMD
        while i + 4 <= a.len() {
            let a_vec = f32x4::from([
                a[i],
                a[i + 1],
                a[i + 2],
                a[i + 3],
            ]);
            
            let b_vec = f32x4::from([
                b[i],
                b[i + 1],
                b[i + 2],
                b[i + 3],
            ]);
            
            sum += a_vec * b_vec;
            i += 4;
        }
        
        // Sum the SIMD results
        let sum_array = sum.to_array();
        let mut dot_product = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements (less than 4)
        while i < a.len() {
            dot_product += a[i] * b[i];
            i += 1;
        }
        
        // Dot product distance = -dot_product
        // (negative because higher dot product = higher similarity)
        Ok(-dot_product)
    }
    
    fn simd_similarity(&self, a: &[f32], b: &[f32]) -> DbResult<f32> {
        if a.len() != b.len() {
            return Err(common::DbError::InvalidOperation(
                "Vector dimensions must match".to_string()
            ));
        }
        
        if a.is_empty() {
            return Ok(0.0);
        }
        
        // Calculate dot product using SIMD
        let mut sum = f32x4::ZERO;
        let mut i = 0;
        
        // Process 4 elements at a time using SIMD
        while i + 4 <= a.len() {
            let a_vec = f32x4::from([
                a[i],
                a[i + 1],
                a[i + 2],
                a[i + 3],
            ]);
            
            let b_vec = f32x4::from([
                b[i],
                b[i + 1],
                b[i + 2],
                b[i + 3],
            ]);
            
            sum += a_vec * b_vec;
            i += 4;
        }
        
        // Sum the SIMD results
        let sum_array = sum.to_array();
        let mut dot_product = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements (less than 4)
        while i < a.len() {
            dot_product += a[i] * b[i];
            i += 1;
        }
        
        // Dot product similarity = dot_product
        Ok(dot_product)
    }
    
    fn name(&self) -> &'static str {
        "SIMD-DotProduct"
    }
}

impl Default for SimdDotProductMetric {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_cosine_metric() {
        let metric = SimdCosineMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.simd_distance(&a, &b).unwrap();
        let similarity = metric.simd_similarity(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
        
        // Test orthogonal vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let distance = metric.simd_distance(&a, &b).unwrap();
        let similarity = metric.simd_similarity(&a, &b).unwrap();
        assert!((distance - 1.0).abs() < f32::EPSILON);
        assert!((similarity - 0.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_simd_euclidean_metric() {
        let metric = SimdEuclideanMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.simd_distance(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        
        // Test unit distance
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 0.0, 0.0];
        let distance = metric.simd_distance(&a, &b).unwrap();
        assert!((distance - 1.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_simd_manhattan_metric() {
        let metric = SimdManhattanMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.simd_distance(&a, &b).unwrap();
        assert!((distance - 0.0).abs() < f32::EPSILON);
        
        // Test unit distance
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 0.0, 0.0];
        let distance = metric.simd_distance(&a, &b).unwrap();
        assert!((distance - 1.0).abs() < f32::EPSILON);
    }
    
    #[test]
    fn test_simd_dot_product_metric() {
        let metric = SimdDotProductMetric::new();
        
        // Test identical vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let distance = metric.simd_distance(&a, &b).unwrap();
        let similarity = metric.simd_similarity(&a, &b).unwrap();
        assert!((distance - (-1.0)).abs() < f32::EPSILON);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
    }
}