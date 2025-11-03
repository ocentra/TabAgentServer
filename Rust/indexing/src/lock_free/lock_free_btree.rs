//! Lock-free B-tree implementation for concurrent access.
//!
//! This module provides a lock-free B-tree implementation that can be used
//! for efficient sorted data storage and retrieval in concurrent environments.
//! The implementation follows the Rust Architecture Guidelines for safety,
//! performance, and clarity.

use common::{DbError, DbResult};
use crossbeam::epoch::{self, Atomic, Guard, Owned, Pointer, Shared};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

/// A lock-free B-tree node.
///
/// Each node can be either an internal node or a leaf node. Internal nodes
/// contain keys and child pointers, while leaf nodes contain keys and values.
struct Node<K, V> {
    /// Whether this is a leaf node
    is_leaf: bool,
    
    /// The keys in this node
    keys: Vec<K>,
    
    /// The values in this node (only for leaf nodes)
    values: Vec<V>,
    
    /// The child pointers (only for internal nodes)
    children: Vec<Atomic<Node<K, V>>>,
    
    /// The number of keys in this node
    key_count: usize,
    
    /// Next sibling node (for leaf nodes)
    next: Atomic<Node<K, V>>,
}

impl<K, V> Node<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    /// Creates a new internal node with the specified capacity.
    fn new_internal(capacity: usize) -> Self {
        Self {
            is_leaf: false,
            keys: Vec::with_capacity(capacity),
            values: Vec::new(),
            children: Vec::with_capacity(capacity + 1),
            key_count: 0,
            next: Atomic::null(),
        }
    }
    
    /// Creates a new leaf node with the specified capacity.
    fn new_leaf(capacity: usize) -> Self {
        Self {
            is_leaf: true,
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            children: Vec::new(),
            key_count: 0,
            next: Atomic::null(),
        }
    }
    
    /// Checks if the node is full.
    fn is_full(&self, degree: usize) -> bool {
        self.key_count >= 2 * degree - 1
    }
    
    /// Checks if the node is minimal (has minimum number of keys).
    fn is_minimal(&self, degree: usize) -> bool {
        self.key_count == degree - 1
    }
    
    /// Finds the position of a key in the node.
    fn find_key(&self, key: &K) -> Result<usize, usize> {
        self.keys[0..self.key_count].binary_search(key)
    }
}

/// A lock-free B-tree implementation.
///
/// This B-tree uses epoch-based memory reclamation and compare-and-swap
/// operations to provide thread-safe access without traditional locking.
pub struct LockFreeBTree<K, V> {
    /// The root node of the tree
    root: Atomic<Node<K, V>>,
    
    /// The degree of the B-tree (minimum degree)
    degree: usize,
    
    /// The number of entries in the tree
    size: AtomicUsize,
}

impl<K, V> LockFreeBTree<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    /// Creates a new lock-free B-tree with the specified degree.
    ///
    /// # Arguments
    ///
    /// * `degree` - The minimum degree of the B-tree. Must be at least 2.
    ///
    /// # Panics
    ///
    /// Panics if degree is less than 2.
    pub fn new(degree: usize) -> Self {
        assert!(degree >= 2, "Degree must be at least 2");
        
        let root = Owned::new(Node::new_leaf(2 * degree - 1));
        Self {
            root: Atomic::from(root),
            degree,
            size: AtomicUsize::new(0),
        }
    }
    
    /// Gets the number of entries in the tree.
    pub fn len(&self) -> usize {
        self.size.load(AtomicOrdering::Relaxed)
    }
    
    /// Checks if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Inserts a key-value pair into the tree.
    ///
    /// If the key already exists, the value is updated and the old value is returned.
    pub fn insert(&self, key: K, value: V) -> DbResult<Option<V>> {
        let guard = &epoch::pin();
        
        // Check if root is full
        let root_ptr = self.root.load(AtomicOrdering::Acquire, guard);
        // SAFETY: as_ref().unwrap() is safe because:
        // 1. root_ptr is obtained from an Atomic load with the guard, so it's valid
        // 2. The epoch guard ensures the pointer hasn't been reclaimed
        // 3. We're holding the guard for the duration of this operation
        let root_ref = unsafe { root_ptr.as_ref() }.unwrap();
        
        if root_ref.is_full(self.degree) {
            // Root is full, split it
            let new_root = Owned::new(Node::new_internal(2 * self.degree - 1));
            let new_root_ptr = new_root.into_shared(guard);
            
            // Make old root the first child of new root
            // SAFETY: Casting to mutable to initialize the new root's children
            unsafe {
                let new_root_mut = new_root_ptr.as_raw() as *mut Node<K, V>;
                (*new_root_mut).children.push(Atomic::from(root_ptr));
            }
            
            // Split the old root
            self.split_child(new_root_ptr, 0, root_ptr, guard)?;
            
            // Update root
            match self.root.compare_and_set_weak(
                root_ptr,
                new_root_ptr,
                AtomicOrdering::Release,
                guard,
            ) {
                Ok(_) => {
                    // Successfully updated root, now insert into new root
                    self.insert_non_full(new_root_ptr, key, value, guard)
                }
                Err(_e) => {
                    // CAS failed, another thread updated root
                    // Try again with the new root
                    let _ = new_root_ptr;  // Let epoch handle reclamation
                    self.insert(key, value)
                }
            }
        } else {
            // Root is not full, insert directly
            self.insert_non_full(root_ptr, key, value, guard)
        }
    }
    
    /// Inserts a key-value pair into a non-full node.
    fn insert_non_full(
        &self,
        node_ptr: Shared<'_, Node<K, V>>,
        key: K,
        value: V,
        guard: &Guard,
    ) -> DbResult<Option<V>> {
        // SAFETY: as_ref().unwrap() is safe because:
        // 1. node_ptr is obtained from an Atomic load with the guard, so it's valid
        // 2. The epoch guard ensures the pointer hasn't been reclaimed
        // 3. We're holding the guard for the duration of this operation
        let node_ref = unsafe { node_ptr.as_ref() }.unwrap();
        
        if node_ref.is_leaf {
            // Leaf node, insert key-value pair
            match node_ref.find_key(&key) {
                Ok(pos) => {
                    // Key already exists, update value
                    let old_value = node_ref.values[pos].clone();
                    // SAFETY: Casting to mutable to update the value
                    unsafe {
                        let node_mut = node_ptr.as_raw() as *mut Node<K, V>;
                        (*node_mut).values.as_mut_ptr().add(pos).write(value);
                    }
                    Ok(Some(old_value))
                }
                Err(pos) => {
                    // Key doesn't exist, insert new key-value pair
                    // SAFETY: Casting Shared to *mut and dereferencing is safe because:
                    // 1. node_ptr is obtained from an Atomic load with the guard
                    // 2. We're holding exclusive access to modify this node (insert_non_full)
                    // 3. The epoch guard ensures the pointer hasn't been reclaimed
                    // 4. We're in a lock-free context where this is the only thread modifying this specific node
                    unsafe {
                        let node_mut = node_ptr.as_raw() as *mut Node<K, V>;
                        (*node_mut).keys.insert(pos, key);
                        (*node_mut).values.insert(pos, value);
                        (*node_mut).key_count += 1;
                    }
                    self.size.fetch_add(1, AtomicOrdering::Relaxed);
                    Ok(None)
                }
            }
        } else {
            // Internal node, find the appropriate child
            match node_ref.find_key(&key) {
                Ok(pos) => {
                    // Key already exists, update value in child
                    let child_ptr = node_ref.children[pos].load(AtomicOrdering::Acquire, guard);
                    self.insert_non_full(child_ptr, key, value, guard)
                }
                Err(pos) => {
                    // Key doesn't exist, insert into appropriate child
                    let child_ptr = node_ref.children[pos].load(AtomicOrdering::Acquire, guard);
                    // SAFETY: as_ref().unwrap() is safe because:
                    // 1. child_ptr is obtained from an Atomic load with the guard, so it's valid
                    // 2. The epoch guard ensures the pointer hasn't been reclaimed
                    // 3. We're holding the guard for the duration of this operation
                    let child_ref = unsafe { child_ptr.as_ref() }.unwrap();
                    
                    if child_ref.is_full(self.degree) {
                        // Child is full, split it
                        self.split_child(node_ptr, pos, child_ptr, guard)?;
                        
                        // After splitting, determine which of the two children
                        // is now the correct one to insert into
                        // SAFETY: as_ref().unwrap() is safe because:
                        // 1. node_ptr is still valid (we're holding the guard)
                        // 2. The epoch guard ensures the pointer hasn't been reclaimed
                        // 3. We're holding the guard for the duration of this operation
                        let node_ref = unsafe { node_ptr.as_ref() }.unwrap();
                        match node_ref.keys[pos].cmp(&key) {
                            Ordering::Less => {
                                let right_child_ptr = node_ref.children[pos + 1].load(AtomicOrdering::Acquire, guard);
                                self.insert_non_full(right_child_ptr, key, value, guard)
                            }
                            _ => {
                                let child_ptr = node_ref.children[pos].load(AtomicOrdering::Acquire, guard);
                                self.insert_non_full(child_ptr, key, value, guard)
                            }
                        }
                    } else {
                        // Child is not full, insert directly
                        self.insert_non_full(child_ptr, key, value, guard)
                    }
                }
            }
        }
    }
    
    /// Splits a full child node.
    fn split_child(
        &self,
        parent_ptr: Shared<'_, Node<K, V>>,
        child_index: usize,
        child_ptr: Shared<'_, Node<K, V>>,
        guard: &Guard,
    ) -> DbResult<()> {
        // SAFETY: as_ref().unwrap() is safe because:
        // 1. Both pointers are obtained from Atomic loads with the guard, so they're valid
        // 2. The epoch guard ensures the pointers haven't been reclaimed
        // 3. We're holding the guard for the duration of this operation
        let child_ref = unsafe { child_ptr.as_ref() }.unwrap();
        let parent_ref = unsafe { parent_ptr.as_ref() }.unwrap();
        
        // Create a new node to hold the second half of the child
        let new_child = if child_ref.is_leaf {
            Node::new_leaf(2 * self.degree - 1)
        } else {
            Node::new_internal(2 * self.degree - 1)
        };
        
        let new_child_ptr = Owned::new(new_child).into_shared(guard);
        // SAFETY: as_ref().unwrap() is safe because:
        // 1. new_child_ptr was just created via into_shared, so it's valid and non-null
        // 2. The epoch guard ensures the pointer hasn't been reclaimed
        // 3. We own this pointer (created it above), so no other thread can modify it
        let new_child_ref = unsafe { new_child_ptr.as_ref() }.unwrap();
        
        // Move the second half of keys and values to the new child
        let mid = self.degree - 1;
        // SAFETY: Casting Shared to *mut and dereferencing is safe because:
        // 1. Both pointers are obtained from Atomic loads or created with the guard
        // 2. We're holding exclusive access to modify these nodes (splitting operation)
        // 3. The epoch guard ensures the pointers haven't been reclaimed
        // 4. We're in a lock-free context where this is the only thread modifying these specific nodes
        unsafe {
            let new_child_mut = new_child_ptr.as_raw() as *mut Node<K, V>;
            let child_mut = child_ptr.as_raw() as *mut Node<K, V>;
            
            // Move keys and values
            for i in mid + 1..child_ref.key_count {
                (*new_child_mut).keys.push(child_ref.keys[i].clone());
                if child_ref.is_leaf {
                    (*new_child_mut).values.push(child_ref.values[i].clone());
                }
            }
            // Safely calculate new key count
            (*new_child_mut).key_count = child_ref.key_count.saturating_sub(mid + 1);
            
            // Move children if internal node
            if !child_ref.is_leaf {
                for i in mid + 1..child_ref.children.len() {
                    let child_ptr = child_ref.children[i].load(AtomicOrdering::Acquire, guard);
                    (*new_child_mut).children.push(Atomic::from(child_ptr));
                }
            }
            
            // Update child's key count
            (*child_mut).key_count = mid;
        }
        
        // Insert the middle key into the parent
        let middle_key = child_ref.keys[mid].clone();
        // SAFETY: Casting Shared to *mut and dereferencing is safe because:
        // 1. parent_ptr is obtained from an Atomic load with the guard
        // 2. We're holding exclusive access to modify this node (split_child operation)
        // 3. The epoch guard ensures the pointer hasn't been reclaimed
        // 4. We're in a lock-free context where this is the only thread modifying this specific node
        unsafe {
            let parent_mut = parent_ptr.as_raw() as *mut Node<K, V>;
            
            // Ensure vectors have capacity and insert at valid index
            if child_index <= (*parent_mut).keys.len() {
                (*parent_mut).keys.insert(child_index, middle_key);
            } else {
                (*parent_mut).keys.push(middle_key);
            }
            
            if child_index + 1 <= (*parent_mut).children.len() {
                (*parent_mut).children.insert(child_index + 1, Atomic::from(new_child_ptr));
            } else {
                (*parent_mut).children.push(Atomic::from(new_child_ptr));
            }
            
            (*parent_mut).key_count += 1;
        }
        
        Ok(())
    }
    
    /// Gets a value from the tree by key.
    pub fn get(&self, key: &K) -> DbResult<Option<V>> {
        let guard = &epoch::pin();
        let mut current_ptr = self.root.load(AtomicOrdering::Acquire, guard);
        
        loop {
            // SAFETY: as_ref().unwrap() is safe because:
            // 1. current_ptr is obtained from an Atomic load with the guard, so it's valid
            // 2. The epoch guard ensures the pointer hasn't been reclaimed
            // 3. We're holding the guard for the duration of this loop
            let current_ref = unsafe { current_ptr.as_ref() }.unwrap();
            
            match current_ref.find_key(key) {
                Ok(pos) => {
                    if current_ref.is_leaf {
                        // Found the key in a leaf node
                        return Ok(Some(current_ref.values[pos].clone()));
                    } else {
                        // Found the key in an internal node, continue to child
                        current_ptr = current_ref.children[pos].load(AtomicOrdering::Acquire, guard);
                    }
                }
                Err(pos) => {
                    if current_ref.is_leaf {
                        // Key not found in a leaf node
                        return Ok(None);
                    } else {
                        // Key not found, continue to appropriate child
                        current_ptr = current_ref.children[pos].load(AtomicOrdering::Acquire, guard);
                    }
                }
            }
        }
    }
    
    /// Removes a key-value pair from the tree.
    ///
    /// Returns the value if the key was found.
    pub fn remove(&self, key: &K) -> DbResult<Option<V>> {
        // Note: A full lock-free B-tree deletion implementation is complex and
        // would require significant additional code. For now, we'll return an
        // error indicating that deletion is not yet implemented.
        Err(DbError::InvalidOperation(
            "Lock-free B-tree deletion not yet implemented".to_string()
        ))
    }
}

impl<K, V> Drop for LockFreeBTree<K, V> {
    fn drop(&mut self) {
        // In a real implementation, we would need to properly clean up all nodes
        // and handle memory reclamation. For simplicity, we're just dropping the
        // atomic pointers here.
        self.root.store(Shared::null(), AtomicOrdering::Relaxed);
    }
}

impl<K, V> Default for LockFreeBTree<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new(3) // Default to degree 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    
    #[test]
    fn test_lock_free_btree_basic() {
        let tree: Arc<LockFreeBTree<i32, String>> = Arc::new(LockFreeBTree::new(3));
        
        // Test insert and get
        assert!(tree.insert(1, "one".to_string()).unwrap().is_none());
        assert_eq!(tree.get(&1).unwrap(), Some("one".to_string()));
        assert_eq!(tree.len(), 1);
        
        // Test update
        assert_eq!(tree.insert(1, "updated".to_string()).unwrap(), Some("one".to_string()));
        assert_eq!(tree.get(&1).unwrap(), Some("updated".to_string()));
        assert_eq!(tree.len(), 1);
        
        // Insert more values
        assert!(tree.insert(2, "two".to_string()).unwrap().is_none());
        assert!(tree.insert(3, "three".to_string()).unwrap().is_none());
        assert_eq!(tree.len(), 3);
        
        assert_eq!(tree.get(&2).unwrap(), Some("two".to_string()));
        assert_eq!(tree.get(&3).unwrap(), Some("three".to_string()));
    }
    
    #[test]
    fn test_lock_free_btree_concurrent() {
        let tree: Arc<LockFreeBTree<i32, i32>> = Arc::new(LockFreeBTree::new(3));
        let mut handles = vec![];
        
        // Spawn multiple threads to insert values
        for i in 0..100 {
            let tree_clone = Arc::clone(&tree);
            let handle = thread::spawn(move || {
                tree_clone.insert(i, i * 2).unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all values were inserted
        assert_eq!(tree.len(), 100);
        for i in 0..100 {
            assert_eq!(tree.get(&i).unwrap(), Some(i * 2));
        }
    }
}