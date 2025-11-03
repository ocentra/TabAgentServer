//! Stress tests for lock-free data structures.
//!
//! This module contains comprehensive stress tests to verify the correctness
//! and performance of the lock-free implementations under highly concurrent
//! workloads.

#[cfg(test)]
mod tests {
    use crate::lock_free::lock_free::{LockFreeHashMap, LockFreeAccessTracker, LockFreeStats};
    use crate::lock_free_hot_vector::LockFreeHotVectorIndex;
    use crate::lock_free_hot_graph::LockFreeHotGraphIndex;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_lock_free_hash_map_stress() {
        const NUM_THREADS: usize = 10;
        const OPERATIONS_PER_THREAD: usize = 1000;
        
        let map: Arc<LockFreeHashMap<String, usize>> = Arc::new(LockFreeHashMap::new(64));
        let mut handles = vec![];
        
        // Spawn multiple threads to perform concurrent operations
        for thread_id in 0..NUM_THREADS {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    let key = format!("thread{}_op{}", thread_id, i);
                    let value = thread_id * OPERATIONS_PER_THREAD + i;
                    
                    // Insert
                    map_clone.insert(key.clone(), value).unwrap();
                    
                    // Get
                    let retrieved = map_clone.get(&key).unwrap();
                    assert_eq!(retrieved, Some(value));
                    
                    // Update
                    map_clone.insert(key.clone(), value + 1).unwrap();
                    
                    // Get updated value
                    let retrieved = map_clone.get(&key).unwrap();
                    assert_eq!(retrieved, Some(value + 1));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final state
        assert_eq!(map.len(), NUM_THREADS * OPERATIONS_PER_THREAD);
        
        // Verify all values
        for thread_id in 0..NUM_THREADS {
            for i in 0..OPERATIONS_PER_THREAD {
                let key = format!("thread{}_op{}", thread_id, i);
                let expected_value = thread_id * OPERATIONS_PER_THREAD + i + 1;
                let retrieved = map.get(&key).unwrap();
                assert_eq!(retrieved, Some(expected_value));
            }
        }
    }
    
    #[test]
    fn test_lock_free_hash_map_concurrent_insert_remove() {
        const NUM_THREADS: usize = 8;
        const KEYS_PER_THREAD: usize = 500;
        
        let map: Arc<LockFreeHashMap<String, usize>> = Arc::new(LockFreeHashMap::new(64));
        let mut insert_handles = vec![];
        let mut remove_handles = vec![];
        
        // Spawn threads to insert keys
        for thread_id in 0..NUM_THREADS {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                for i in 0..KEYS_PER_THREAD {
                    let key = format!("insert_thread{}_key{}", thread_id, i);
                    let value = thread_id * KEYS_PER_THREAD + i;
                    map_clone.insert(key, value).unwrap();
                }
            });
            insert_handles.push(handle);
        }
        
        // Spawn threads to remove keys (with some delay to ensure insertion happens first)
        thread::sleep(Duration::from_millis(10));
        
        for thread_id in 0..NUM_THREADS {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                for i in 0..(KEYS_PER_THREAD / 2) {
                    let key = format!("insert_thread{}_key{}", thread_id, i);
                    map_clone.remove(&key).unwrap();
                }
            });
            remove_handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in insert_handles {
            handle.join().unwrap();
        }
        for handle in remove_handles {
            handle.join().unwrap();
        }
        
        // Verify that at least some keys remain
        assert!(map.len() > 0);
    }
    
    #[test]
    fn test_lock_free_access_tracker_stress() {
        const NUM_THREADS: usize = 10;
        const OPERATIONS_PER_THREAD: usize = 1000;
        
        let tracker = Arc::new(LockFreeAccessTracker::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to record accesses
        for _ in 0..NUM_THREADS {
            let tracker_clone = Arc::clone(&tracker);
            let handle = thread::spawn(move || {
                for _ in 0..OPERATIONS_PER_THREAD {
                    tracker_clone.record_access();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final count
        assert_eq!(tracker.access_count(), (NUM_THREADS * OPERATIONS_PER_THREAD) as u64);
    }
    
    #[test]
    fn test_lock_free_stats_stress() {
        const NUM_THREADS: usize = 10;
        const OPERATIONS_PER_THREAD: usize = 1000;
        
        let stats = Arc::new(LockFreeStats::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to update statistics
        for _ in 0..NUM_THREADS {
            let stats_clone = Arc::clone(&stats);
            let handle = thread::spawn(move || {
                for _ in 0..OPERATIONS_PER_THREAD {
                    stats_clone.increment_query_count();
                    stats_clone.increment_similarity_computations();
                    stats_clone.add_query_time(10);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final counts
        assert_eq!(stats.query_count.load(std::sync::atomic::Ordering::Relaxed), 
                   NUM_THREADS * OPERATIONS_PER_THREAD);
        assert_eq!(stats.similarity_computations.load(std::sync::atomic::Ordering::Relaxed), 
                   NUM_THREADS * OPERATIONS_PER_THREAD);
        assert_eq!(stats.total_query_time_micros.load(std::sync::atomic::Ordering::Relaxed), 
                   (NUM_THREADS * OPERATIONS_PER_THREAD * 10) as u64);
    }
    
    #[test]
    fn test_lock_free_hot_vector_index_stress() {
        const NUM_THREADS: usize = 8;
        const VECTORS_PER_THREAD: usize = 200;
        const VECTOR_DIMENSION: usize = 128;
        
        let index = Arc::new(LockFreeHotVectorIndex::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to add vectors
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let handle = thread::spawn(move || {
                for i in 0..VECTORS_PER_THREAD {
                    let vector_id = format!("thread{}_vector{}", thread_id, i);
                    let vector = vec![((thread_id * VECTORS_PER_THREAD + i) as f32 / 1000.0); VECTOR_DIMENSION];
                    index_clone.add_vector(&vector_id, vector).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final count
        assert_eq!(index.len(), NUM_THREADS * VECTORS_PER_THREAD);
    }
    
    #[test]
    fn test_lock_free_hot_graph_index_stress() {
        const NUM_THREADS: usize = 6;
        const NODES_PER_THREAD: usize = 100;
        const EDGES_PER_THREAD: usize = 200;
        
        let graph = Arc::new(LockFreeHotGraphIndex::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to add nodes
        for thread_id in 0..NUM_THREADS {
            let graph_clone = Arc::clone(&graph);
            let handle = thread::spawn(move || {
                // Add nodes
                for i in 0..NODES_PER_THREAD {
                    let node_id = format!("thread{}_node{}", thread_id, i);
                    graph_clone.add_node(&node_id, None).unwrap();
                }
                
                // Add edges
                for i in 0..EDGES_PER_THREAD {
                    let from_node = format!("thread{}_node{}", thread_id, i % NODES_PER_THREAD);
                    let to_node = format!("thread{}_node{}", thread_id, (i + 1) % NODES_PER_THREAD);
                    graph_clone.add_edge(&from_node, &to_node).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final count
        assert_eq!(graph.node_count(), NUM_THREADS * NODES_PER_THREAD);
    }
    
    #[test]
    fn test_lock_free_mixed_operations_stress() {
        const NUM_THREADS: usize = 8;
        const OPERATIONS_PER_THREAD: usize = 500;
        
        let vector_index = Arc::new(LockFreeHotVectorIndex::new());
        let graph_index = Arc::new(LockFreeHotGraphIndex::new());
        let mut handles = vec![];
        
        // Spawn multiple threads to perform mixed operations
        for thread_id in 0..NUM_THREADS {
            let vector_index_clone = Arc::clone(&vector_index);
            let graph_index_clone = Arc::clone(&graph_index);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    // Alternate between vector and graph operations
                    if i % 2 == 0 {
                        // Vector operations
                        let vector_id = format!("thread{}_vector{}", thread_id, i);
                        let vector = vec![((thread_id * OPERATIONS_PER_THREAD + i) as f32 / 1000.0); 64];
                        vector_index_clone.add_vector(&vector_id, vector).unwrap();
                    } else {
                        // Graph operations
                        let node_id = format!("thread{}_node{}", thread_id, i);
                        graph_index_clone.add_node(&node_id, None).unwrap();
                        
                        if i >= 2 {
                            let prev_node_id = format!("thread{}_node{}", thread_id, i - 2);
                            if i % 4 == 1 {
                                graph_index_clone.add_edge(&prev_node_id, &node_id).unwrap();
                            }
                        }
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final counts
        // Note: The exact counts depend on the operations performed
        assert!(vector_index.len() > 0);
        assert!(graph_index.node_count() > 0);
    }
    
    #[test]
    fn test_lock_free_concurrent_read_write() {
        const WRITER_THREADS: usize = 4;
        const READER_THREADS: usize = 6;
        const OPERATIONS_PER_WRITER: usize = 500;
        const OPERATIONS_PER_READER: usize = 1000;
        
        let map: Arc<LockFreeHashMap<String, usize>> = Arc::new(LockFreeHashMap::new(64));
        let mut handles = vec![];
        
        // Spawn writer threads
        for thread_id in 0..WRITER_THREADS {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_WRITER {
                    let key = format!("writer{}_op{}", thread_id, i);
                    let value = thread_id * OPERATIONS_PER_WRITER + i;
                    map_clone.insert(key, value).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Spawn reader threads
        for thread_id in 0..READER_THREADS {
            let map_clone = Arc::clone(&map);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_READER {
                    let key_index = (thread_id * OPERATIONS_PER_READER + i) % (WRITER_THREADS * OPERATIONS_PER_WRITER);
                    let writer_id = key_index / OPERATIONS_PER_WRITER;
                    let op_id = key_index % OPERATIONS_PER_WRITER;
                    let key = format!("writer{}_op{}", writer_id, op_id);
                    
                    // Try to read - it's okay if it doesn't exist yet
                    let _ = map_clone.get(&key);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final count
        assert_eq!(map.len(), WRITER_THREADS * OPERATIONS_PER_WRITER);
    }
}
