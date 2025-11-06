/// Common types and utilities for lock-free data structures
use std::sync::atomic::{AtomicUsize, Ordering};

/// Access tracker for temperature-based management
#[derive(Debug)]
pub struct LockFreeAccessTracker {
    access_count: AtomicUsize,
    last_access: AtomicUsize,
}

impl LockFreeAccessTracker {
    /// Create a new access tracker
    pub fn new() -> Self {
        Self {
            access_count: AtomicUsize::new(0),
            last_access: AtomicUsize::new(0),
        }
    }

    /// Record an access
    pub fn record_access(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        // In a real implementation, update timestamp
    }

    /// Get access count
    pub fn access_count(&self) -> usize {
        self.access_count.load(Ordering::Relaxed)
    }
}

impl Default for LockFreeAccessTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for lock-free data structures
#[derive(Debug)]
pub struct LockFreeStats {
    pub(crate) vector_count: AtomicUsize,
    pub(crate) query_count: AtomicUsize,
    pub(crate) promotions: AtomicUsize,
    pub(crate) demotions: AtomicUsize,
    pub(crate) total_query_time_micros: AtomicUsize,
    pub(crate) similarity_computations: AtomicUsize,
}

impl LockFreeStats {
    /// Create new statistics tracker
    pub fn new() -> Self {
        Self {
            vector_count: AtomicUsize::new(0),
            query_count: AtomicUsize::new(0),
            promotions: AtomicUsize::new(0),
            demotions: AtomicUsize::new(0),
            total_query_time_micros: AtomicUsize::new(0),
            similarity_computations: AtomicUsize::new(0),
        }
    }

    pub fn increment_vector_count(&self) {
        self.vector_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_vector_count(&self) {
        self.vector_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn increment_query_count(&self) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_promotions(&self) {
        self.promotions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_demotions(&self) {
        self.demotions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_similarity_computations(&self) {
        self.similarity_computations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_query_time(&self, micros: u64) {
        self.total_query_time_micros.fetch_add(micros as usize, Ordering::Relaxed);
    }

    pub fn vector_count(&self) -> usize {
        self.vector_count.load(Ordering::Relaxed)
    }

    pub fn query_count(&self) -> usize {
        self.query_count.load(Ordering::Relaxed)
    }

    pub fn promotions(&self) -> usize {
        self.promotions.load(Ordering::Relaxed)
    }

    pub fn demotions(&self) -> usize {
        self.demotions.load(Ordering::Relaxed)
    }
}

impl Default for LockFreeStats {
    fn default() -> Self {
        Self::new()
    }
}

