//! # KvStore, a Key-Value Store
//!
//! Implementation of redis-like key-value store
//! as part of [PingCAP Talent Plan](https://github.com/pingcap/talent-plan).
//!
//! Nothing fancy, but should allow you to [KvStore::set], [KvStore::get] and [KvStore::remove]
//! in a in-memory cache.
#![warn(missing_docs)]
use std::collections::HashMap;
use std::hash::Hash;

/// KvStore is a key-value store allowing you store values in-memory with O(1) lookup time.
/// # Example
/// ```
/// use kvs::KvStore;
/// let mut store = KvStore::new();
///
/// store.set("key1", "value1");
/// assert_eq!(Some("value1"), store.get("key1"));
///
/// store.remove("key1");
/// assert_eq!(None, store.get("key1"));
/// ```
#[derive(Debug)]
pub struct KvStore<K, V> {
    store: HashMap<K, V>,
}

impl<K: Eq + Hash, V: Clone> KvStore<K, V> {
    /// Initialize new store
    pub fn new() -> Self {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// Set a value. Overrides the value if key is already present
    pub fn set(&mut self, key: K, value: V) {
        self.store.insert(key, value);
    }

    /// Get a value.
    pub fn get(&self, key: K) -> Option<V> {
        self.store.get(&key).map(|value| value.clone())
    }

    /// Remove a value. If value wasn't present, nothing happens.
    pub fn remove(&mut self, key: K) {
        self.store.remove(&key);
    }
}
