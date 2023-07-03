use crate::store::Command;
use crate::store::KiwiEngine;
use crate::{Error, Result};

use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct KiwiStoreInner {
    write_log: File,
    full_path: PathBuf,
    store: HashMap<String, u64>,
}

impl KiwiStoreInner {
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

        Ok(KiwiStoreInner {
            write_log,
            full_path,
            store,
        })
    }

    /// Set a value. Overrides the value if key is already present
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let offset = self.write_log.seek(SeekFrom::End(0))?;

        // trigger compaction if file is ~4000 entries long
        if offset > 4000 * 21 {
            self.compact()?;
        }

        self.store.insert(key.clone(), offset);
        let command = serde_json::to_string(&Command::Set((key, value))).unwrap();
        self.write_log.write_all((command + "\n").as_bytes())?;
        Ok(())
    }

    /// Get a value.
    fn get(&self, key: String) -> Result<Option<String>> {
        match self.store.get(&key) {
            Some(offset) => Ok(Some(value_from_file(&self.full_path, *offset)?)),
            None => Ok(None),
        }
    }

    /// Remove a value. If value wasn't present, nothing happens.
    fn remove(&mut self, key: String) -> Result<()> {
        match self.store.get(&key) {
            Some(_) => {
                self.store.remove(&key);
                let command = serde_json::to_string(&Command::Remove(key)).unwrap();
                self.write_log.write_all((command + "\n").as_bytes())?;
                Ok(())
            }
            None => Err(Error::NoKey(String::from("Key not found"))),
        }
    }

    fn compact(&mut self) -> Result<()> {
        // open new file kvs.db.tmp
        let path = self.full_path.clone();
        let tmp_path = self.full_path.clone().with_extension(".tmp");

        let mut new_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&tmp_path)?;
        let mut new_offset = 0u64;

        // for each key in self.store
        for (key, offset) in self.store.iter_mut() {
            // save current value as Command::Set to the new file
            let value = value_from_file(&path, *offset)?;
            let command = serde_json::to_string(&Command::Set((key.clone(), value)))?;
            let offset_change = new_log.write((command + "\n").as_bytes())?;
            // update key offset
            *offset = new_offset;
            new_offset += offset_change as u64;
        }

        // replace db file with the temporary one
        fs::rename(&tmp_path, &path)?;

        // update write_log file
        self.write_log = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(())
    }
}

/// KvStore is a key-value store allowing you store values in-memory with O(1) lookup time.
/// # Example
/// ```
/// # use std::error::Error;
/// # use tempfile::TempDir;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// # let some_dir = TempDir::new().unwrap();
/// use kiwi_store::{KiwiStore, KiwiEngine};
/// let mut store = KiwiStore::open(some_dir.path())?;
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
pub struct KiwiStore {
    inner: Arc<RwLock<KiwiStoreInner>>,
}

impl KiwiStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(KiwiStore {
            inner: Arc::new(RwLock::new(KiwiStoreInner::open(path)?)),
        })
    }
}

impl KiwiEngine for KiwiStore {
    /// Set a value. Overrides the value if key is already present
    fn set(&self, key: String, value: String) -> Result<()> {
        self.inner
            .write()
            .expect("error acquiring lock")
            .set(key, value)
    }

    /// Get a value.
    fn get(&self, key: String) -> Result<Option<String>> {
        self.inner.read().expect("error acquiring lock").get(key)
    }

    /// Remove a value. If value wasn't present, nothing happens.
    fn remove(&self, key: String) -> Result<()> {
        self.inner
            .write()
            .expect("error acquiring lock")
            .remove(key)
    }
}

fn value_from_file(path: &Path, offset: u64) -> Result<String> {
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
