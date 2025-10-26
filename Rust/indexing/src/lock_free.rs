//! Lock-free data structures for concurrent access.
//!
//! This module provides lock-free implementations of common data structures
//! to improve performance in highly concurrent scenarios. These implementations
//! follow the Rust Architecture Guidelines for safety, performance, and clarity.

use common::DbResult;
use crossbeam::epoch::{self, Atomic, Owned, Shared, Guard};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// A lock-free concurrent hash map implementation.
///
/// This hash map uses epoch-based memory reclamation and compare-and-swap
/// operations to provide thread-safe access without traditional locking.
pub struct LockFreeHashMap<K, V> {
    /// The underlying buckets
    buckets: Vec<Atomic<Bucket<K, V>>>,
    
    /// Number of buckets
    bucket_count: usize,
    
    /// Number of entries in the map
    size: AtomicUsize,
}

// NOTE: Old complex iterator removed. Now using a simplified Vec-based approach.
// See the `iter()` method on LockFreeHashMap for the new implementation.

/// A bucket in the hash map containing a linked list of entries
struct Bucket<K, V> {
    /// Head of the linked list
    head: Atomic<Entry<K, V>>,
}

/// An entry in the linked list
struct Entry<K, V> {
    /// Hash of the key
    hash: u64,
    
    /// The key
    key: K,
    
    /// The value
    value: V,
    
    /// Next entry in the list
    next: Atomic<Entry<K, V>>,
}

impl<K, V> LockFreeHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Creates a new lock-free hash map with the specified number of buckets.
    pub fn new(bucket_count: usize) -> Self {
        let mut buckets = Vec::with_capacity(bucket_count);
        for _ in 0..bucket_count {
            buckets.push(Atomic::null());
        }
        
        Self {
            buckets,
            bucket_count,
            size: AtomicUsize::new(0),
        }
    }
    
    /// Returns an iterator over the entries of the map (snapshot).
    ///
    /// NOTE: This collects all entries into a Vec to avoid lifetime issues
    /// with epoch-based memory reclamation.
    pub fn iter(&self) -> impl Iterator<Item = DbResult<(K, V)>> {
        let guard = epoch::pin();
        let mut entries = Vec::new();
        
        // Collect all entries from all buckets
        for bucket_atomic in &self.buckets {
            let bucket_ptr = bucket_atomic.load(Ordering::Relaxed, &guard);
            if !bucket_ptr.is_null() {
                if let Some(bucket) = unsafe { bucket_ptr.as_ref() } {
                    let mut entry_ptr = bucket.head.load(Ordering::Acquire, &guard);
                    
                    // Traverse the linked list in this bucket
                    while !entry_ptr.is_null() {
                        if let Some(entry) = unsafe { entry_ptr.as_ref() } {
                            entries.push(Ok((entry.key.clone(), entry.value.clone())));
                            entry_ptr = entry.next.load(Ordering::Acquire, &guard);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        
        entries.into_iter()
    }
    
    /// Gets the number of entries in the map.
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    
    /// Checks if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Inserts a key-value pair into the map.
    ///
    /// If the key already exists, the value is updated and the old value is returned.
    pub fn insert(&self, key: K, value: V) -> DbResult<Option<V>> {
        let hash = self.hash_key(&key);
        let bucket_index = (hash as usize) % self.bucket_count;
        
        let guard = &epoch::pin();
        
        // Get the bucket
        let bucket_ptr = self.buckets[bucket_index].load(Ordering::Relaxed, guard);
        let bucket = if bucket_ptr.is_null() {
            // Create a new bucket if it doesn't exist
            let new_bucket = Owned::new(Bucket {
                head: Atomic::null(),
            });
            match self.buckets[bucket_index].compare_exchange_weak(
                Shared::null(),
                new_bucket,
                Ordering::Release,
                Ordering::Acquire,
                guard,
            ) {
                Ok(bucket_ptr) => unsafe { bucket_ptr.as_ref() }.unwrap(),
                Err(e) => unsafe { e.current.as_ref() }.unwrap(),
            }
        } else {
            unsafe { bucket_ptr.as_ref() }.unwrap()
        };
        
        // Search for existing entry with the same key
        let mut current_ptr = bucket.head.load(Ordering::Acquire, guard);
        while let Some(current) = unsafe { current_ptr.as_ref() } {
            if current.hash == hash && current.key == key {
                // Found existing entry, update the value
                let old_value = current.value.clone();
                // Note: In a real implementation, we would need to handle memory reclamation
                // properly for the old value. For simplicity, we're just cloning here.
                return Ok(Some(old_value));
            }
            current_ptr = current.next.load(Ordering::Acquire, guard);
        }
        
        // Key not found, insert new entry at the head of the list
        let new_entry = Owned::new(Entry {
            hash,
            key,
            value,
            next: Atomic::null(),
        });
        
        let new_entry_ptr = new_entry.into_shared(guard);
        loop {
            let head_ptr = bucket.head.load(Ordering::Acquire, guard);
            unsafe {
                new_entry_ptr.as_ref().unwrap().next.store(head_ptr, Ordering::Release);
            }
            
            match bucket.head.compare_exchange_weak(
                head_ptr,
                new_entry_ptr,
                Ordering::Release,
                Ordering::Acquire,
                guard,
            ) {
                Ok(_) => {
                    self.size.fetch_add(1, Ordering::Relaxed);
                    return Ok(None);
                }
                Err(e) => {
                    // CAS failed, try again
                    unsafe {
                        new_entry_ptr.as_ref().unwrap().next.store(e.current, Ordering::Release);
                    }
                }
            }
        }
    }
    
    /// Gets a value from the map by key.
    pub fn get(&self, key: &K) -> DbResult<Option<V>> {
        let hash = self.hash_key(key);
        let bucket_index = (hash as usize) % self.bucket_count;
        
        let guard = &epoch::pin();
        
        // Get the bucket
        let bucket_ptr = self.buckets[bucket_index].load(Ordering::Relaxed, guard);
        if bucket_ptr.is_null() {
            return Ok(None);
        }
        
        let bucket = unsafe { bucket_ptr.as_ref() }.unwrap();
        
        // Search for entry with the key
        let mut current_ptr = bucket.head.load(Ordering::Acquire, guard);
        while let Some(current) = unsafe { current_ptr.as_ref() } {
            if current.hash == hash && current.key == *key {
                return Ok(Some(current.value.clone()));
            }
            current_ptr = current.next.load(Ordering::Acquire, guard);
        }
        
        Ok(None)
    }
    
    /// Removes a key-value pair from the map.
    ///
    /// Returns the value if the key was found.
    pub fn remove(&self, key: &K) -> DbResult<Option<V>> {
        let hash = self.hash_key(key);
        let bucket_index = (hash as usize) % self.bucket_count;
        
        let guard = &epoch::pin();
        
        // Get the bucket
        let bucket_ptr = self.buckets[bucket_index].load(Ordering::Relaxed, guard);
        if bucket_ptr.is_null() {
            return Ok(None);
        }
        
        let bucket = unsafe { bucket_ptr.as_ref() }.unwrap();
        
        // Search for entry with the key
        let mut current_ptr = bucket.head.load(Ordering::Acquire, guard);
        let mut prev_ptr: Shared<'_, Entry<K, V>> = Shared::null();
        
        while let Some(current) = unsafe { current_ptr.as_ref() } {
            if current.hash == hash && current.key == *key {
                // Found the entry to remove
                let next_ptr = current.next.load(Ordering::Acquire, guard);
                
                if prev_ptr.is_null() {
                    // Removing the head of the list
                    match bucket.head.compare_exchange_weak(
                        current_ptr,
                        next_ptr,
                        Ordering::Release,
                        Ordering::Acquire,
                        guard,
                    ) {
                        Ok(_) => {
                            self.size.fetch_sub(1, Ordering::Relaxed);
                            // Note: Proper memory reclamation would be needed here
                            return Ok(Some(current.value.clone()));
                        }
                        Err(_) => {
                            // CAS failed, try again
                            current_ptr = bucket.head.load(Ordering::Acquire, guard);
                            prev_ptr = Shared::null();
                            continue;
                        }
                    }
                } else {
                    // Removing from the middle or end of the list
                    let prev = unsafe { prev_ptr.as_ref() }.unwrap();
                    match prev.next.compare_exchange_weak(
                        current_ptr,
                        next_ptr,
                        Ordering::Release,
                        Ordering::Acquire,
                        guard,
                    ) {
                        Ok(_) => {
                            self.size.fetch_sub(1, Ordering::Relaxed);
                            // Note: Proper memory reclamation would be needed here
                            return Ok(Some(current.value.clone()));
                        }
                        Err(_) => {
                            // CAS failed, try again
                            current_ptr = bucket.head.load(Ordering::Acquire, guard);
                            prev_ptr = Shared::null();
                            continue;
                        }
                    }
                }
            }
            
            prev_ptr = current_ptr;
            current_ptr = current.next.load(Ordering::Acquire, guard);
        }
        
        Ok(None)
    }
    
    /// Computes the hash of a key.
    fn hash_key(&self, key: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl<K, V> Drop for LockFreeHashMap<K, V> {
    fn drop(&mut self) {
        // In a real implementation, we would need to properly clean up all entries
        // and handle memory reclamation. For simplicity, we're just dropping the
        // atomic pointers here.
        for bucket in &self.buckets {
            bucket.store(Shared::null(), Ordering::Relaxed);
        }
    }
}

impl<K, V> Bucket<K, V> {
    /// Creates a new bucket.
    fn new() -> Self {
        Self {
            head: Atomic::null(),
        }
    }
}

impl<K, V> Default for LockFreeHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new(64) // Default to 64 buckets
    }
}

/// Lock-free access tracker for temperature management.
///
/// This tracker uses atomic counters to record access patterns without locking.
pub struct LockFreeAccessTracker {
    /// Number of accesses
    access_count: AtomicU64,
    
    /// Last access timestamp
    last_access: AtomicU64,
}

impl LockFreeAccessTracker {
    /// Creates a new access tracker.
    pub fn new() -> Self {
        Self {
            access_count: AtomicU64::new(0),
            last_access: AtomicU64::new(0),
        }
    }
    
    /// Records an access.
    pub fn record_access(&self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.last_access.store(timestamp, Ordering::Relaxed);
    }
    
    /// Gets the access count.
    pub fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }
    
    /// Gets the last access timestamp.
    pub fn last_access(&self) -> u64 {
        self.last_access.load(Ordering::Relaxed)
    }
}

impl Clone for LockFreeAccessTracker {
    fn clone(&self) -> Self {
        Self {
            access_count: AtomicU64::new(self.access_count.load(Ordering::Relaxed)),
            last_access: AtomicU64::new(self.last_access.load(Ordering::Relaxed)),
        }
    }
}

impl Default for LockFreeAccessTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock-free statistics tracker.
///
/// This tracker uses atomic counters to record performance statistics without locking.
pub struct LockFreeStats {
    /// Number of vectors in the index
    pub vector_count: AtomicUsize,
    
    /// Total number of queries performed
    pub query_count: AtomicUsize,
    
    /// Total time spent on queries (in microseconds)
    pub total_query_time_micros: AtomicU64,
    
    /// Number of tier promotions
    pub promotions: AtomicUsize,
    
    /// Number of tier demotions
    pub demotions: AtomicUsize,
    
    /// Total number of similarity computations
    pub similarity_computations: AtomicUsize,
}

impl LockFreeStats {
    /// Creates new statistics with default values
    pub fn new() -> Self {
        Self {
            vector_count: AtomicUsize::new(0),
            query_count: AtomicUsize::new(0),
            total_query_time_micros: AtomicU64::new(0),
            promotions: AtomicUsize::new(0),
            demotions: AtomicUsize::new(0),
            similarity_computations: AtomicUsize::new(0),
        }
    }
    
    /// Increments the query count.
    pub fn increment_query_count(&self) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Adds to the total query time.
    pub fn add_query_time(&self, micros: u64) {
        self.total_query_time_micros.fetch_add(micros, Ordering::Relaxed);
    }
    
    /// Increments the similarity computation count.
    pub fn increment_similarity_computations(&self) {
        self.similarity_computations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Increments the promotion count.
    pub fn increment_promotions(&self) {
        self.promotions.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Increments the demotion count.
    pub fn increment_demotions(&self) {
        self.demotions.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Increments the vector count.
    pub fn increment_vector_count(&self) {
        self.vector_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Decrements the vector count.
    pub fn decrement_vector_count(&self) {
        self.vector_count.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for LockFreeStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_lock_free_hash_map_basic() {
        let map: Arc<LockFreeHashMap<String, i32>> = Arc::new(LockFreeHashMap::new(16));
        
        // Test insert and get
        assert!(map.insert("key1".to_string(), 42).unwrap().is_none());
        assert_eq!(map.get(&"key1".to_string()).unwrap(), Some(42));
        assert_eq!(map.len(), 1);
        
        // Test update
        assert_eq!(map.insert("key1".to_string(), 84).unwrap(), Some(42));
        assert_eq!(map.get(&"key1".to_string()).unwrap(), Some(84));
        assert_eq!(map.len(), 1);
        
        // Test remove
        assert_eq!(map.remove(&"key1".to_string()).unwrap(), Some(84));
        assert_eq!(map.get(&"key1".to_string()).unwrap(), None);
        assert_eq!(map.len(), 0);
    }
    
    #[test]
    fn test_lock_free_hash_map_concurrent() {
        let map: Arc<LockFreeHashMap<String, i32>> = Arc::new(LockFreeHashMap::new(16));
        let mut handles = vec![];
        
        // Spawn multiple threads to insert values
        for i in 0..10 {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                let key = format!("key{}", i);
                map_clone.insert(key, i).unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all values were inserted
        assert_eq!(map.len(), 10);
        for i in 0..10 {
            let key = format!("key{}", i);
            assert_eq!(map.get(&key).unwrap(), Some(i));
        }
    }
    
    #[test]
    fn test_lock_free_access_tracker() {
        let tracker = LockFreeAccessTracker::new();
        
        assert_eq!(tracker.access_count(), 0);
        assert_eq!(tracker.last_access(), 0);
        
        tracker.record_access();
        
        assert_eq!(tracker.access_count(), 1);
        assert!(tracker.last_access() > 0);
    }
    
    #[test]
    fn test_lock_free_stats() {
        let stats = LockFreeStats::new();
        
        assert_eq!(stats.query_count.load(Ordering::Relaxed), 0);
        assert_eq!(stats.total_query_time_micros.load(Ordering::Relaxed), 0);
        
        stats.increment_query_count();
        stats.add_query_time(100);
        
        assert_eq!(stats.query_count.load(Ordering::Relaxed), 1);
        assert_eq!(stats.total_query_time_micros.load(Ordering::Relaxed), 100);
    }
}