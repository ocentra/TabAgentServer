//! Adaptive concurrency control that switches between lock-free and traditional
//! locking based on system load.
//!
//! This module provides adaptive concurrency control mechanisms that can
//! automatically switch between lock-free and traditional locking implementations
//! based on system load, contention levels, and performance metrics.
//!
//! The implementation follows the Rust Architecture Guidelines for safety,
//! performance, and clarity.

use crate::hybrid::{HotVectorIndex, HotGraphIndex};
use crate::lock_free_hot_vector::LockFreeHotVectorIndex;
use crate::lock_free_hot_graph::LockFreeHotGraphIndex;
use common::{DbError, DbResult};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::time::{Instant, Duration};

/// Concurrency control mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrencyMode {
    /// Use traditional locking (Mutex/RwLock)
    Traditional,
    
    /// Use lock-free data structures
    LockFree,
}

/// Performance metrics for adaptive concurrency control.
#[derive(Debug, Clone)]
pub struct ConcurrencyMetrics {
    /// Number of operations in the current window
    pub operation_count: usize,
    
    /// Average operation latency in microseconds
    pub avg_latency_micros: u64,
    
    /// Contention level (0-100)
    pub contention_level: u32,
    
    /// Memory usage in bytes
    pub memory_usage: usize,
}

/// Adaptive concurrency controller.
pub struct AdaptiveConcurrencyController {
    /// Current concurrency mode
    mode: ConcurrencyMode,
    
    /// Performance metrics
    metrics: Arc<Mutex<ConcurrencyMetrics>>,
    
    /// Threshold for switching to lock-free mode (operations per second)
    lock_free_threshold: usize,
    
    /// Threshold for switching back to traditional mode (operations per second)
    traditional_threshold: usize,
    
    /// Last switch time
    last_switch: Instant,
    
    /// Minimum time between switches (in seconds)
    min_switch_interval: u64,
    
    /// Whether adaptive switching is enabled
    adaptive_enabled: AtomicBool,
}

impl AdaptiveConcurrencyController {
    /// Creates a new adaptive concurrency controller.
    pub fn new() -> Self {
        Self {
            mode: ConcurrencyMode::Traditional,
            metrics: Arc::new(Mutex::new(ConcurrencyMetrics {
                operation_count: 0,
                avg_latency_micros: 0,
                contention_level: 0,
                memory_usage: 0,
            })),
            lock_free_threshold: 10000, // 10K operations per second
            traditional_threshold: 1000, // 1K operations per second
            last_switch: Instant::now(),
            min_switch_interval: 30, // 30 seconds
            adaptive_enabled: AtomicBool::new(true),
        }
    }
    
    /// Creates a new adaptive concurrency controller with custom thresholds.
    pub fn with_thresholds(
        lock_free_threshold: usize,
        traditional_threshold: usize,
        min_switch_interval: u64,
    ) -> Self {
        Self {
            mode: ConcurrencyMode::Traditional,
            metrics: Arc::new(Mutex::new(ConcurrencyMetrics {
                operation_count: 0,
                avg_latency_micros: 0,
                contention_level: 0,
                memory_usage: 0,
            })),
            lock_free_threshold,
            traditional_threshold,
            last_switch: Instant::now(),
            min_switch_interval,
            adaptive_enabled: AtomicBool::new(true),
        }
    }
    
    /// Gets the current concurrency mode.
    pub fn get_mode(&self) -> ConcurrencyMode {
        self.mode
    }
    
    /// Sets the concurrency mode manually.
    pub fn set_mode(&mut self, mode: ConcurrencyMode) {
        self.mode = mode;
        self.last_switch = Instant::now();
    }
    
    /// Enables or disables adaptive switching.
    pub fn set_adaptive_enabled(&self, enabled: bool) {
        self.adaptive_enabled.store(enabled, Ordering::Relaxed);
    }
    
    /// Updates performance metrics.
    pub fn update_metrics(&self, metrics: ConcurrencyMetrics) {
        if let Ok(mut guard) = self.metrics.lock() {
            *guard = metrics;
        }
    }
    
    /// Checks if a mode switch is needed based on current metrics.
    pub fn check_mode_switch(&mut self) -> Option<ConcurrencyMode> {
        if !self.adaptive_enabled.load(Ordering::Relaxed) {
            return None;
        }
        
        // Check if enough time has passed since last switch
        if self.last_switch.elapsed() < Duration::from_secs(self.min_switch_interval) {
            return None;
        }
        
        if let Ok(metrics) = self.metrics.lock() {
            match self.mode {
                ConcurrencyMode::Traditional => {
                    // Switch to lock-free if load is high
                    if metrics.operation_count >= self.lock_free_threshold {
                        self.mode = ConcurrencyMode::LockFree;
                        self.last_switch = Instant::now();
                        return Some(ConcurrencyMode::LockFree);
                    }
                }
                ConcurrencyMode::LockFree => {
                    // Switch back to traditional if load is low
                    if metrics.operation_count <= self.traditional_threshold {
                        self.mode = ConcurrencyMode::Traditional;
                        self.last_switch = Instant::now();
                        return Some(ConcurrencyMode::Traditional);
                    }
                }
            }
        }
        
        None
    }
}

impl Default for AdaptiveConcurrencyController {
    fn default() -> Self {
        Self::new()
    }
}

/// Adaptive vector index that can switch between implementations.
pub struct AdaptiveVectorIndex {
    /// Traditional implementation
    traditional: Arc<Mutex<HotVectorIndex>>,
    
    /// Lock-free implementation
    lock_free: Arc<LockFreeHotVectorIndex>,
    
    /// Concurrency controller
    controller: Arc<Mutex<AdaptiveConcurrencyController>>,
    
    /// Operation counter for metrics
    operation_counter: AtomicUsize,
}

impl AdaptiveVectorIndex {
    /// Creates a new adaptive vector index.
    pub fn new() -> Self {
        Self {
            traditional: Arc::new(Mutex::new(HotVectorIndex::new())),
            lock_free: Arc::new(LockFreeHotVectorIndex::new()),
            controller: Arc::new(Mutex::new(AdaptiveConcurrencyController::new())),
            operation_counter: AtomicUsize::new(0),
        }
    }
    
    /// Adds a vector to the index.
    pub fn add_vector(&self, id: &str, vector: Vec<f32>) -> DbResult<()> {
        self.increment_operation_counter();
        
        if let Ok(controller) = self.controller.lock() {
            match controller.get_mode() {
                ConcurrencyMode::Traditional => {
                    if let Ok(mut index) = self.traditional.lock() {
                        return index.add_vector(id, vector);
                    }
                }
                ConcurrencyMode::LockFree => {
                    return self.lock_free.add_vector(id, vector);
                }
            }
        }
        
        Err(DbError::InvalidOperation(
            "Failed to acquire lock for vector index operation".to_string()
        ))
    }
    
    /// Removes a vector from the index.
    pub fn remove_vector(&self, id: &str) -> DbResult<bool> {
        self.increment_operation_counter();
        
        if let Ok(controller) = self.controller.lock() {
            match controller.get_mode() {
                ConcurrencyMode::Traditional => {
                    if let Ok(mut index) = self.traditional.lock() {
                        return index.remove_vector(id);
                    }
                }
                ConcurrencyMode::LockFree => {
                    return self.lock_free.remove_vector(id);
                }
            }
        }
        
        Err(DbError::InvalidOperation(
            "Failed to acquire lock for vector index operation".to_string()
        ))
    }
    
    /// Searches for similar vectors.
    pub fn search(&self, query: &[f32], k: usize) -> DbResult<Vec<(String, f32)>> {
        self.increment_operation_counter();
        
        if let Ok(controller) = self.controller.lock() {
            match controller.get_mode() {
                ConcurrencyMode::Traditional => {
                    if let Ok(mut index) = self.traditional.lock() {
                        return index.search(query, k);
                    }
                }
                ConcurrencyMode::LockFree => {
                    return self.lock_free.search(query, k);
                }
            }
        }
        
        Err(DbError::InvalidOperation(
            "Failed to acquire lock for vector index operation".to_string()
        ))
    }
    
    /// Gets the number of vectors in the index.
    pub fn len(&self) -> usize {
        // For simplicity, we'll return the length from the traditional implementation
        // In a real implementation, both implementations should be kept in sync
        if let Ok(index) = self.traditional.lock() {
            index.len()
        } else {
            0
        }
    }
    
    /// Checks if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Increments the operation counter and checks for mode switch.
    fn increment_operation_counter(&self) {
        self.operation_counter.fetch_add(1, Ordering::Relaxed);
        
        // Periodically check if we should switch modes
        if self.operation_counter.load(Ordering::Relaxed) % 1000 == 0 {
            if let Ok(mut controller) = self.controller.lock() {
                controller.check_mode_switch();
            }
        }
    }
    
    /// Gets the current concurrency mode.
    pub fn get_mode(&self) -> ConcurrencyMode {
        if let Ok(controller) = self.controller.lock() {
            controller.get_mode()
        } else {
            ConcurrencyMode::Traditional
        }
    }
    
    /// Sets the concurrency mode manually.
    pub fn set_mode(&self, mode: ConcurrencyMode) {
        if let Ok(mut controller) = self.controller.lock() {
            controller.set_mode(mode);
        }
    }
}

impl Default for AdaptiveVectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Adaptive graph index that can switch between implementations.
pub struct AdaptiveGraphIndex {
    /// Traditional implementation
    traditional: Arc<RwLock<HotGraphIndex>>,
    
    /// Lock-free implementation
    lock_free: Arc<LockFreeHotGraphIndex>,
    
    /// Concurrency controller
    controller: Arc<Mutex<AdaptiveConcurrencyController>>,
}

impl AdaptiveGraphIndex {
    /// Creates a new adaptive graph index.
    pub fn new() -> Self {
        Self {
            traditional: Arc::new(RwLock::new(HotGraphIndex::new())),
            lock_free: Arc::new(LockFreeHotGraphIndex::new()),
            controller: Arc::new(Mutex::new(AdaptiveConcurrencyController::new())),
        }
    }
    
    /// Adds a node to the graph.
    pub fn add_node(&self, id: &str, metadata: Option<&str>) -> DbResult<()> {
        if let Ok(controller) = self.controller.lock() {
            match controller.get_mode() {
                ConcurrencyMode::Traditional => {
                    if let Ok(mut graph) = self.traditional.write() {
                        return graph.add_node(id, metadata);
                    }
                }
                ConcurrencyMode::LockFree => {
                    return self.lock_free.add_node(id, metadata);
                }
            }
        }
        
        Err(DbError::InvalidOperation(
            "Failed to acquire lock for graph index operation".to_string()
        ))
    }
    
    /// Adds an edge to the graph.
    pub fn add_edge(&self, from: &str, to: &str) -> DbResult<()> {
        if let Ok(controller) = self.controller.lock() {
            match controller.get_mode() {
                ConcurrencyMode::Traditional => {
                    if let Ok(mut graph) = self.traditional.write() {
                        return graph.add_edge(from, to);
                    }
                }
                ConcurrencyMode::LockFree => {
                    return self.lock_free.add_edge(from, to);
                }
            }
        }
        
        Err(DbError::InvalidOperation(
            "Failed to acquire lock for graph index operation".to_string()
        ))
    }
    
    /// Gets the current concurrency mode.
    pub fn get_mode(&self) -> ConcurrencyMode {
        if let Ok(controller) = self.controller.lock() {
            controller.get_mode()
        } else {
            ConcurrencyMode::Traditional
        }
    }
    
    /// Sets the concurrency mode manually.
    pub fn set_mode(&self, mode: ConcurrencyMode) {
        if let Ok(mut controller) = self.controller.lock() {
            controller.set_mode(mode);
        }
    }
}

impl Default for AdaptiveGraphIndex {
    fn default() -> Self {
        Self::new()
    }
}
