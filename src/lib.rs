//! # KvStore, a CLI Key-Value Store
//!
//! Implementation of command-line redis-like key-value store
//! as part of [PingCAP Talent Plan](https://github.com/pingcap/talent-plan).
//!
//! Nothing fancy, but should allow you to [KvStore::set], [KvStore::get] and [KvStore::remove]
//! in a in-memory cache.
#![warn(missing_docs)]

use std::path::PathBuf;
use std::error::Error;
use std::result;
use std::fs::{File, OpenOptions};
use std::io::Write;
use serde::{Serialize, Deserialize};


// TODO(tkarwwoski): might use https://docs.rs/fehler/1.0.0/fehler/ instead
pub type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set((String, String)),
    Remove(String),
}

/// KvStore is a key-value store allowing you store values in-memory with O(1) lookup time.
/// # Example
/// ```
/// use kvs::KvStore;
/// let mut store = KvStore::open();
///
/// store.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(Some("value1".to_owned()), store.get("key1".to_owned()));
///
/// store.remove("key1".to_owned());
/// assert_eq!(None, store.get("key1".to_owned()));
/// ```
#[derive(Debug)]
pub struct KvStore {
    log: File,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path.into())
            .unwrap();
        Ok(KvStore { log: file })
    }

    /// Set a value. Overrides the value if key is already present
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = serde_json::to_string(&Command::Set((key, value))).unwrap();
        self.log.write((command + "\n").as_bytes())?;
        Ok(())
    }

    /// Get a value.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        unimplemented!()
    }

    /// Remove a value. If value wasn't present, nothing happens.
    pub fn remove(&mut self, key: String) -> Result<()> {
        let command = serde_json::to_string(&Command::Remove(key)).unwrap();
        self.log.write((command + "\n").as_bytes())?;
        Ok(())
    }
}
