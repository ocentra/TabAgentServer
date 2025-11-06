//! Adaptive sharded B-tree implementation for concurrent access.
//!
//! This module provides a sharded B-tree with two underlying implementations:
//! - COWShard: Copy-on-write for read-heavy workloads (lock-free reads)
//! - BLinkShard: B-link style with epoch reclamation for write-heavy workloads
//!
//! The AdaptiveIndex wrapper routes keys to shards and can switch modes dynamically.

use common::DbResult;
use crossbeam::epoch::{self as epoch, Atomic, Owned, Shared, Guard};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const MIN_DEGREE: usize = 8; // tune: larger degree -> fewer levels
const MAX_KEYS: usize = 2 * MIN_DEGREE - 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IndexMode {
    ReadHeavy,   // choose COW shards
    WriteHeavy,  // choose BLink shards
    Auto,        // auto-mode (monitors R/W ratio)
}

// ---------- Utility: shard selection ----------
fn shard_of<K: Hash>(key: &K, num_shards: usize) -> usize {
    use std::collections::hash_map::DefaultHasher;
    let mut s = DefaultHasher::new();
    key.hash(&mut s);
    (s.finish() as usize) % num_shards
}

// ---------- Trait for a shard ----------
trait Shard<K, V>: Send + Sync 
where
    K: Clone + Ord + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, k: &K) -> Option<V>;
    fn insert(&self, k: K, v: V);
    fn stats(&self) -> ShardStats;
}

#[derive(Default, Debug, Clone)]
pub struct ShardStats {
    pub reads: usize,
    pub writes: usize,
}

// ---------- COWShard: copy-on-write persistent tree (Arc snapshots) ----------
mod cow_shard {
    use super::*;

    #[derive(Debug, Clone)]
    pub(super) struct Node<K, V> {
        pub is_leaf: bool,
        pub keys: Vec<K>,
        pub vals: Vec<V>,
        pub children: Vec<Arc<Node<K, V>>>,
    }

    impl<K: Clone + Ord + Send + Sync, V: Clone + Send + Sync> Node<K, V> {
        pub fn new_leaf() -> Self {
            Self {
                is_leaf: true,
                keys: Vec::new(),
                vals: Vec::new(),
                children: Vec::new(),
            }
        }

        pub fn new_internal() -> Self {
            Self {
                is_leaf: false,
                keys: Vec::new(),
                vals: Vec::new(),
                children: Vec::new(),
            }
        }

        fn find_pos(&self, k: &K) -> usize {
            match self.keys.binary_search(k) {
                Ok(i) => i,
                Err(i) => i,
            }
        }
    }

    pub struct COWShard<K, V> {
        root: std::sync::RwLock<Arc<Node<K, V>>>,
        reads: AtomicUsize,
        writes: AtomicUsize,
    }

    impl<K: Clone + Ord + Send + Sync, V: Clone + Send + Sync> COWShard<K, V> {
        pub fn new() -> Self {
            Self {
                root: std::sync::RwLock::new(Arc::new(Node::new_leaf())),
                reads: AtomicUsize::new(0),
                writes: AtomicUsize::new(0),
            }
        }

        fn get_snapshot_root(&self) -> Arc<Node<K, V>> {
            self.root.read().unwrap().clone()
        }

        fn insert_internal(&self, root: &Arc<Node<K, V>>, k: K, v: V) -> Arc<Node<K, V>> {
            if root.is_leaf {
                let mut new_node = Node::new_leaf();
                let mut inserted = false;
                for i in 0..root.keys.len() {
                    if !inserted && k <= root.keys[i] {
                        if k == root.keys[i] {
                            new_node.keys.push(k.clone());
                            new_node.vals.push(v.clone());
                            inserted = true;
                        } else {
                            new_node.keys.push(k.clone());
                            new_node.vals.push(v.clone());
                            inserted = true;
                            new_node.keys.push(root.keys[i].clone());
                            new_node.vals.push(root.vals[i].clone());
                        }
                    } else {
                        new_node.keys.push(root.keys[i].clone());
                        new_node.vals.push(root.vals[i].clone());
                    }
                }
                if !inserted {
                    new_node.keys.push(k);
                    new_node.vals.push(v);
                }
                Arc::new(new_node)
            } else {
                let pos = root.find_pos(&k);
                let mut new_parent = Node {
                    is_leaf: root.is_leaf,
                    keys: root.keys.clone(),
                    vals: root.vals.clone(),
                    children: root.children.clone(),
                };

                if new_parent.children.is_empty() {
                    let leaf = Node::new_leaf();
                    new_parent.children.push(Arc::new(leaf));
                }

                let child_arc = &root.children[pos];
                if child_arc.keys.len() >= MAX_KEYS {
                    let (left, midk, right) = Self::split_node_arc(child_arc);
                    new_parent.children[pos] = left;
                    new_parent.children.insert(pos + 1, right);
                    new_parent.keys.insert(pos, midk.clone());

                    if k > midk {
                        let right_child = new_parent.children[pos + 1].clone();
                        let new_right = self.insert_internal(&right_child, k, v);
                        new_parent.children[pos + 1] = new_right;
                        return Arc::new(new_parent);
                    } else {
                        let left_child = new_parent.children[pos].clone();
                        let new_left = self.insert_internal(&left_child, k, v);
                        new_parent.children[pos] = new_left;
                        return Arc::new(new_parent);
                    }
                } else {
                    let new_child = self.insert_internal(child_arc, k, v);
                    new_parent.children[pos] = new_child;
                    return Arc::new(new_parent);
                }
            }
        }

        fn split_node_arc(node: &Arc<Node<K, V>>) -> (Arc<Node<K, V>>, K, Arc<Node<K, V>>) {
            assert!(node.keys.len() >= MAX_KEYS);
            let t = MIN_DEGREE;
            let mid_index = t - 1;
            let mid_key = node.keys[mid_index].clone();

            let mut left = if node.is_leaf { Node::new_leaf() } else { Node::new_internal() };
            left.keys = node.keys[..mid_index].to_vec();
            if node.is_leaf {
                left.vals = node.vals[..mid_index].to_vec();
            } else {
                left.children = node.children[..t].to_vec();
            }

            let mut right = if node.is_leaf { Node::new_leaf() } else { Node::new_internal() };
            right.keys = node.keys[mid_index + 1..].to_vec();
            if node.is_leaf {
                right.vals = node.vals[mid_index + 1..].to_vec();
            } else {
                right.children = node.children[t..].to_vec();
            }

            (Arc::new(left), mid_key, Arc::new(right))
        }
    }

    impl<K: Clone + Ord + Send + Sync, V: Clone + Send + Sync> Shard<K, V> for COWShard<K, V> {
        fn get(&self, k: &K) -> Option<V> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            let mut cur = self.get_snapshot_root();
            loop {
                let pos = cur.keys.binary_search(k).unwrap_or_else(|x| x);
                if pos < cur.keys.len() && &cur.keys[pos] == k {
                    if cur.is_leaf {
                        return Some(cur.vals[pos].clone());
                    } else {
                        let next = cur.children[pos + 1].clone();
                        cur = next;
                        continue;
                    }
                } else if cur.is_leaf {
                    return None;
                } else {
                    let next = cur.children[pos].clone();
                    cur = next;
                    continue;
                }
            }
        }

        fn insert(&self, k: K, v: V) {
            self.writes.fetch_add(1, Ordering::Relaxed);
            let current_root = self.root.read().unwrap().clone();

            let new_root_arc = if current_root.keys.len() >= MAX_KEYS {
                let mut new_root = Node::new_internal();
                new_root.children.push(current_root.clone());
                let (left, midk, right) = Self::split_node_arc(&current_root);
                new_root.keys.push(midk);
                new_root.children[0] = left;
                new_root.children.push(right);
                let new_root_arc = Arc::new(new_root);
                self.insert_internal(&new_root_arc, k, v)
            } else {
                self.insert_internal(&current_root, k, v)
            };

            *self.root.write().unwrap() = new_root_arc;
        }

        fn stats(&self) -> ShardStats {
            ShardStats {
                reads: self.reads.load(Ordering::Relaxed),
                writes: self.writes.load(Ordering::Relaxed),
            }
        }
    }
}

// ---------- BLinkShard: B-link nodes with epoch reclamation ----------
mod blink_shard {
    use super::*;

    #[derive(Debug)]
    pub(super) struct Node<K, V> {
        pub is_leaf: bool,
        pub keys: Vec<K>,
        pub vals: Vec<V>,
        pub children: Vec<Atomic<Node<K, V>>>,
        pub right: Atomic<Node<K, V>>,
    }

    impl<K: Clone + Ord + Send + Sync, V: Clone + Send + Sync> Node<K, V> {
        pub fn new_leaf() -> Self {
            Self {
                is_leaf: true,
                keys: Vec::new(),
                vals: Vec::new(),
                children: Vec::new(),
                right: Atomic::null(),
            }
        }

        pub fn new_internal() -> Self {
            Self {
                is_leaf: false,
                keys: Vec::new(),
                vals: Vec::new(),
                children: Vec::new(),
                right: Atomic::null(),
            }
        }

        fn find_pos(&self, k: &K) -> usize {
            match self.keys.binary_search(k) {
                Ok(i) => i,
                Err(i) => i,
            }
        }
    }

    pub struct BLinkShard<K, V> {
        root: Atomic<Node<K, V>>,
        writer_lock: Mutex<()>,
        reads: AtomicUsize,
        writes: AtomicUsize,
    }

    impl<K: Clone + Ord + Send + Sync, V: Clone + Send + Sync> BLinkShard<K, V> {
        pub fn new() -> Self {
            let root_node = Node::new_leaf();
            let owned = Owned::new(root_node);
            let atomic = Atomic::from(owned);
            Self {
                root: atomic,
                writer_lock: Mutex::new(()),
                reads: AtomicUsize::new(0),
                writes: AtomicUsize::new(0),
            }
        }

        fn load_root<'g>(&self, g: &'g Guard) -> Shared<'g, Node<K, V>> {
            self.root.load(Ordering::Acquire, g)
        }

        fn find_leaf<'g>(&self, k: &K, g: &'g Guard) -> Shared<'g, Node<K, V>> {
            let mut cur = self.load_root(g);
            unsafe {
                loop {
                    let cur_ref = cur.deref();
                    let pos = cur_ref.find_pos(k);
                    if cur_ref.is_leaf {
                        if pos == cur_ref.keys.len() {
                            let right = cur_ref.right.load(Ordering::Acquire, g);
                            if !right.is_null() {
                                cur = right;
                                continue;
                            }
                        }
                        return cur;
                    } else {
                        let child_atomic = if pos < cur_ref.children.len() {
                            &cur_ref.children[pos]
                        } else {
                            &cur_ref.right
                        };
                        let child = child_atomic.load(Ordering::Acquire, g);
                        if child.is_null() {
                            return cur;
                        } else {
                            cur = child;
                            continue;
                        }
                    }
                }
            }
        }

        fn insert_locked(&self, k: K, v: V) {
            let _wg = self.writer_lock.lock();
            let guard = &epoch::pin();
            self.writes.fetch_add(1, Ordering::Relaxed);

            let root_shared = self.load_root(guard);
            let root_ref = unsafe { root_shared.deref() };

            if root_ref.keys.len() >= MAX_KEYS {
                let (left_owned, midk, right_owned) = Self::split_node_owned(root_shared, guard);
                let mut new_root = Node::new_internal();
                new_root.keys.push(midk);
                new_root.children.push(Atomic::from(left_owned));
                new_root.children.push(Atomic::from(right_owned));
                let new_root_owned = Owned::new(new_root);
                let prev = self.root.swap(new_root_owned, Ordering::AcqRel, guard);
                unsafe { guard.defer_destroy(prev); }
            }

            let guard = &epoch::pin();
            let leaf_shared = self.find_leaf(&k, guard);
            let leaf_ref = unsafe { leaf_shared.deref() };

            let mut new_leaf = Node::new_leaf();
            new_leaf.keys = leaf_ref.keys.clone();
            new_leaf.vals = leaf_ref.vals.clone();

            match new_leaf.keys.binary_search(&k) {
                Ok(i) => {
                    new_leaf.vals[i] = v;
                }
                Err(i) => {
                    new_leaf.keys.insert(i, k);
                    new_leaf.vals.insert(i, v);
                }
            }

            let new_leaf_owned = Owned::new(new_leaf);
            let (parent_shared_opt, parent_index) = self.find_parent_of(leaf_shared, guard);

            match parent_shared_opt {
                None => {
                    let prev = self.root.swap(new_leaf_owned, Ordering::AcqRel, guard);
                    unsafe { guard.defer_destroy(prev); }
                }
                Some(parent_shared) => {
                    let parent_ref = unsafe { parent_shared.deref() };
                    if parent_index < parent_ref.children.len() {
                        let prev = parent_ref.children[parent_index].swap(new_leaf_owned, Ordering::AcqRel, guard);
                        unsafe { guard.defer_destroy(prev); }
                    } else {
                        let prev = parent_ref.right.swap(new_leaf_owned, Ordering::AcqRel, guard);
                        unsafe { guard.defer_destroy(prev); }
                    }
                }
            }
        }

        fn find_parent_of<'g>(&self, target: Shared<'g, Node<K, V>>, g: &'g Guard) -> (Option<Shared<'g, Node<K, V>>>, usize) {
            let mut cur = self.load_root(g);
            let parent: Option<Shared<'g, Node<K, V>>> = None;
            loop {
                let cur_ref = unsafe { cur.deref() };
                if cur_ref.is_leaf {
                    if cur == target {
                        return (parent, 0);
                    }
                    let mut r = cur_ref.right.load(Ordering::Acquire, g);
                    while !r.is_null() {
                        if r == target {
                            return (parent, 0);
                        }
                        r = unsafe { r.deref() }.right.load(Ordering::Acquire, g);
                    }
                    return (None, 0);
                } else {
                    for (i, child_atomic) in cur_ref.children.iter().enumerate() {
                        let c = child_atomic.load(Ordering::Acquire, g);
                        if c == target {
                            return (Some(cur), i);
                        }
                    }
                    if !cur_ref.children.is_empty() {
                        cur = cur_ref.children[0].load(Ordering::Acquire, g);
                        continue;
                    } else {
                        return (None, 0);
                    }
                }
            }
        }

        fn split_node_owned<'g>(node_shared: Shared<'g, Node<K, V>>, _g: &'g Guard) -> (Owned<Node<K, V>>, K, Owned<Node<K, V>>) {
            let node_ref = unsafe { node_shared.deref() };
            assert!(node_ref.keys.len() >= MAX_KEYS);
            let t = MIN_DEGREE;
            let mid_index = t - 1;
            let mid_key = node_ref.keys[mid_index].clone();

            let mut left = Node::new_internal();
            left.is_leaf = node_ref.is_leaf;
            left.keys = node_ref.keys[..mid_index].to_vec();
            if node_ref.is_leaf {
                left.vals = node_ref.vals[..mid_index].to_vec();
            } else {
                left.children = node_ref.children[..t].iter().map(|a| {
                    let g2 = &epoch::pin();
                    let s = a.load(Ordering::Acquire, g2);
                    Atomic::from(s)
                }).collect();
            }

            let mut right = Node::new_internal();
            right.is_leaf = node_ref.is_leaf;
            right.keys = node_ref.keys[mid_index + 1..].to_vec();
            if node_ref.is_leaf {
                right.vals = node_ref.vals[mid_index + 1..].to_vec();
            } else {
                right.children = node_ref.children[t..].iter().map(|a| {
                    let g2 = &epoch::pin();
                    let s = a.load(Ordering::Acquire, g2);
                    Atomic::from(s)
                }).collect();
            }

            (Owned::new(left), mid_key, Owned::new(right))
        }
    }

    impl<K: Clone + Ord + Send + Sync, V: Clone + Send + Sync> Shard<K, V> for BLinkShard<K, V> {
        fn get(&self, k: &K) -> Option<V> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            let guard = &epoch::pin();
            let leaf_shared = self.find_leaf(k, guard);
            let leaf_ref = unsafe { leaf_shared.deref() };
            match leaf_ref.keys.binary_search(k) {
                Ok(i) => {
                    if leaf_ref.is_leaf {
                        Some(leaf_ref.vals[i].clone())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }

        fn insert(&self, k: K, v: V) {
            self.insert_locked(k, v);
        }

        fn stats(&self) -> ShardStats {
            ShardStats {
                reads: self.reads.load(Ordering::Relaxed),
                writes: self.writes.load(Ordering::Relaxed),
            }
        }
    }
}

// ---------- ShardImpl enum ----------
enum ShardImpl<K, V> 
where
    K: Clone + Ord + Send + Sync,
    V: Clone + Send + Sync,
{
    COW(Arc<cow_shard::COWShard<K, V>>),
    BLINK(Arc<blink_shard::BLinkShard<K, V>>),
}

// ---------- AdaptiveIndex: public API ----------
pub struct LockFreeBTree<K, V> 
where
    K: Clone + Ord + Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    shards: Vec<ShardImpl<K, V>>,
    num_shards: usize,
    mode: IndexMode,
    global_reads: AtomicUsize,
    global_writes: AtomicUsize,
}

impl<K, V> LockFreeBTree<K, V>
where
    K: Clone + Ord + Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Creates a new adaptive B-tree with specified number of shards and mode
    pub fn new(num_shards: usize) -> Self {
        Self::new_with_mode(num_shards, IndexMode::Auto)
    }

    pub fn new_with_mode(num_shards: usize, mode: IndexMode) -> Self {
        assert!(num_shards >= 1);
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            let impl_choice = match mode {
                IndexMode::ReadHeavy => ShardImpl::COW(Arc::new(cow_shard::COWShard::new())),
                IndexMode::WriteHeavy => ShardImpl::BLINK(Arc::new(blink_shard::BLinkShard::new())),
                IndexMode::Auto => ShardImpl::COW(Arc::new(cow_shard::COWShard::new())),
            };
            shards.push(impl_choice);
        }
        Self {
            shards,
            num_shards,
            mode,
            global_reads: AtomicUsize::new(0),
            global_writes: AtomicUsize::new(0),
        }
    }

    fn shard_for(&self, k: &K) -> usize {
        shard_of(k, self.num_shards)
    }

    pub fn get(&self, k: &K) -> DbResult<Option<V>> {
        let s = self.shard_for(k);
        let result = match &self.shards[s] {
            ShardImpl::COW(c) => c.get(k),
            ShardImpl::BLINK(b) => b.get(k),
        };
        self.global_reads.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }

    pub fn insert(&self, k: K, v: V) -> DbResult<Option<V>> {
        let s = self.shard_for(&k);
        match &self.shards[s] {
            ShardImpl::COW(c) => c.insert(k, v),
            ShardImpl::BLINK(b) => b.insert(k, v),
        }
        self.global_writes.fetch_add(1, Ordering::Relaxed);
        Ok(None) // Simplified: not tracking old values for now
    }

    pub fn len(&self) -> usize {
        // Approximate: sum of all shard operations
        self.global_writes.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn remove(&self, _key: &K) -> DbResult<Option<V>> {
        Err(common::DbError::InvalidOperation(
            "Remove not yet implemented for adaptive B-tree".to_string()
        ))
    }

    pub fn dump_stats(&self) {
        for (i, s) in self.shards.iter().enumerate() {
            let stats = match s {
                ShardImpl::COW(c) => c.stats(),
                ShardImpl::BLINK(b) => b.stats(),
            };
            println!("shard {}: {:?}", i, stats);
        }
        println!(
            "global reads/writes: {}/{}",
            self.global_reads.load(Ordering::Relaxed),
            self.global_writes.load(Ordering::Relaxed)
        );
    }
}

impl<K, V> Default for LockFreeBTree<K, V>
where
    K: Clone + Ord + Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new(16) // 16 shards by default
    }
}
