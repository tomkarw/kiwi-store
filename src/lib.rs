//! # KvStore, a CLI Key-Value Store
//!
//! Implementation of command-line redis-like key-value store
//! as part of [PingCAP Talent Plan](https://github.com/pingcap/talent-plan).
//!
//! Nothing fancy, but should allow you to [KvStore::set], [KvStore::get] and [KvStore::remove]
//! in a in-memory cache.
// #![warn(missing_docs)]
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use sled::Db;

pub use err::{Error, Result};
pub use thread_pool::*;

/// Reexport Result and Error
mod err;
/// Reexport ThreadPool and it's implementations
mod thread_pool;

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set((String, String)),
    Remove(String),
}

#[derive(Debug)]
pub struct KvStoreInner {
    write_log: File,
    full_path: PathBuf,
    store: HashMap<String, u64>,
}

/// Provides a generic set of actions extracted from KvStore
pub trait KvsEngine {
    fn set(&self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<Option<String>>;
    fn remove(&self, key: String) -> Result<()>;
}

/// KvStore is a key-value store allowing you store values in-memory with O(1) lookup time.
/// # Example
/// ```
/// # use std::error::Error;
/// # use tempfile::TempDir;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// # let some_dir = TempDir::new().unwrap();
/// use kvs::{KvStore, KvsEngine};
/// let mut store = KvStore::open(some_dir.path())?;
///
/// store.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(Some("value1".to_owned()), store.get("key1".to_owned())?);
///
/// store.remove("key1".to_owned());
/// assert_eq!(None, store.get("key1".to_owned())?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct KvStore {
    inner: Arc<Mutex<KvStoreInner>>,
}

impl KvStore {
    /// Open KvStore at a specified location.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let mut full_path = path.into();
        full_path.push("kvs.db");

        let mut store = HashMap::new();
        if full_path.exists() {
            let file = File::open(&full_path)?;
            let mut reader = BufReader::new(file);
            let mut buffer = String::new();
            let mut current_offset = 0;

            loop {
                let read_bytes = reader.read_line(&mut buffer)?;
                if read_bytes == 0 {
                    break; // end of stream
                }

                match serde_json::from_str(&buffer)? {
                    Command::Set((key, _)) => {
                        store.insert(key, current_offset as u64);
                    }
                    Command::Remove(key) => {
                        store.remove(&key);
                    }
                };

                buffer.clear();
                current_offset += read_bytes;
            }
        }

        let write_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)?;

        Ok(KvStore {
            inner: Arc::new(Mutex::new(KvStoreInner {
                write_log,
                full_path,
                store,
            })),
        })
    }

    fn value(path: &Path, offset: u64) -> Result<String> {
        let mut file = File::open(path)?;

        file.seek(SeekFrom::Start(offset))?;
        let mut reader = BufReader::new(&file);
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        match serde_json::from_str(&buffer)? {
            Command::Remove(_) => panic!("wrong offset"),
            Command::Set((_, value)) => Ok(value),
        }
    }

    fn compact(&self) -> Result<()> {
        let mut inner = self.inner.lock().expect("error acquiring lock");

        // open new file kvs.db.tmp
        let path = inner.full_path.clone();
        let tmp_path = inner.full_path.clone().with_extension(".tmp");

        let mut new_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&tmp_path)?;
        let mut new_offset = 0u64;

        // for each key in self.store
        for (key, offset) in inner.store.iter_mut() {
            // save current value as Command::Set to the new file
            let value = KvStore::value(&path, *offset)?;
            let command = serde_json::to_string(&Command::Set((key.clone(), value)))?;
            let offset_change = new_log.write((command + "\n").as_bytes())?;
            // update key offset
            *offset = new_offset;
            new_offset += offset_change as u64;
        }

        // replace db file with the temporary one
        fs::rename(&tmp_path, &path)?;

        // update write_log file
        inner.write_log = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(())
    }
}

impl KvsEngine for KvStore {
    /// Set a value. Overrides the value if key is already present
    fn set(&self, key: String, value: String) -> Result<()> {
        let mut inner = self.inner.lock().expect("error acquiring lock");

        let offset = inner.write_log.seek(SeekFrom::End(0))?;

        // trigger compaction if file is ~4000 entries long
        if offset > 4000 * 22 {
            self.compact()?;
        }

        inner.store.insert(key.clone(), offset);
        let command = serde_json::to_string(&Command::Set((key, value))).unwrap();
        inner.write_log.write_all((command + "\n").as_bytes())?;
        Ok(())
    }

    /// Get a value.
    fn get(&self, key: String) -> Result<Option<String>> {
        let inner = self.inner.lock().expect("error acquiring lock");
        match inner.store.get(&key) {
            Some(offset) => Ok(Some(KvStore::value(&inner.full_path, *offset)?)),
            None => Ok(None),
        }
    }

    /// Remove a value. If value wasn't present, nothing happens.
    fn remove(&self, key: String) -> Result<()> {
        let mut inner = self.inner.lock().expect("error acquiring lock");
        match inner.store.get(&key) {
            Some(_) => {
                inner.store.remove(&key);
                let command = serde_json::to_string(&Command::Remove(key)).unwrap();
                inner.write_log.write_all((command + "\n").as_bytes())?;
                Ok(())
            }
            None => Err(Error::NoKey(String::from("Key not found"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SledKvsEngine {
    inner: Arc<Mutex<SledKvsEngineInner>>,
}

#[derive(Debug, Clone)]
pub struct SledKvsEngineInner {
    db: Db,
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(SledKvsEngine {
            inner: Arc::new(Mutex::new(SledKvsEngineInner {
                db: sled::open(path.into())?,
            })),
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        let inner = self.inner.lock().expect("error acquiring lock");
        match inner.db.insert(key.as_bytes(), value.as_bytes()) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::Sled(error)),
        }
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let inner = self.inner.lock().expect("error acquiring lock");
        match inner.db.get(key.as_bytes()) {
            Ok(result) => match result {
                Some(value) => Ok(Some(str::from_utf8(&value.to_vec())?.to_owned())),
                None => Ok(None),
            },
            Err(error) => Err(Error::Sled(error)),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        let inner = self.inner.lock().expect("error acquiring lock");
        match inner.db.remove(key.as_bytes()) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::Sled(error)),
        }
    }
}
