//! # KvStore, a CLI Key-Value Store
//!
//! Implementation of command-line redis-like key-value store
//! as part of [PingCAP Talent Plan](https://github.com/pingcap/talent-plan).
//!
//! Nothing fancy, but should allow you to [KvStore::set], [KvStore::get] and [KvStore::remove]
//! in a in-memory cache.
// #![warn(missing_docs)]

use std::{error, fmt, fs, io, result};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str;

use serde::{Deserialize, Serialize};
use serde::export::Formatter;
use sled::Db;

/// Result specific for this crate, for now it's error case is `Box<dyn Error>` but this might change
// TODO(tkarwowski): might use https://docs.rs/fehler/1.0.0/fehler/ instead
pub type Result<T> = result::Result<T, Error>;

/// Errors possible, [`NoKey`] is KvStore specific,
/// the rest is simply propagated from lower functions
#[derive(Debug)]
pub enum Error {
    /// Error when trying to remove non-existing key
    NoKey(String),
    /// Error when Seek fails due to file corruption
    Offset(String),
    /// Error when any of the IO operation fails
    Io(io::Error),
    /// Error when deserialization failed due to file corruption
    InvalidData(serde_json::Error),
    /// Error when parsing utf-8 to string
    Utf8Error(str::Utf8Error),
    /// Error passed from Sled implementation of KvsEngine
    Sled(sled::Error),
    /// Any ad hoc error
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoKey(msg) => write!(f, "{}", msg),
            Error::Offset(msg) => write!(f, "{}", msg),
            Error::Io(msg) => write!(f, "{}", msg),
            Error::InvalidData(msg) => write!(f, "{}", msg),
            Error::Utf8Error(msg) => write!(f, "{}", msg),
            Error::Sled(msg) => write!(f, "{}", msg),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::InvalidData(err)
    }
}

impl From<sled::Error> for Error {
    fn from(err: sled::Error) -> Self {
        Error::Sled(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        Error::Utf8Error(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set((String, String)),
    Remove(String),
}

/// Provides a generic set of actions extracted from KvStore
pub trait KvsEngine {

    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn remove(&mut self, key: String) -> Result<()>;
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
#[derive(Debug)]
pub struct KvStore {
    write_log: File,
    full_path: PathBuf,
    store: HashMap<String, u64>,
}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        todo!()
    }
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
            write_log,
            full_path,
            store,
        })
    }

    fn value(path: &PathBuf, offset: u64) -> Result<String> {
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
        self.write_log = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(())
    }
}

impl KvsEngine for KvStore {
    /// Get a value.
    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.store.get(&key) {
            Some(offset) => Ok(Some(KvStore::value(&self.full_path, *offset)?)),
            None => Ok(None),
        }
    }

    /// Set a value. Overrides the value if key is already present
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let offset = self.write_log.seek(SeekFrom::End(0))?;

        // trigger compaction if file is ~4000 entries long
        if offset > 4000 * 22 {
            self.compact()?;
        }

        self.store.insert(key.clone(), offset);
        let command = serde_json::to_string(&Command::Set((key, value))).unwrap();
        self.write_log.write_all((command + "\n").as_bytes())?;
        Ok(())
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
}

#[derive(Debug, Clone)]
pub struct SledKvsEngine {
    db: Db,
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(SledKvsEngine {
            db: sled::open(path.into())?,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.db.get(key.as_bytes()) {
            Ok(result) => match result {
                Some(value) => Ok(Some(str::from_utf8(&value.to_vec())?.to_owned())),
                None => Ok(None),
            },
            Err(error) => Err(Error::Sled(error)),
        }
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        match self.db.insert(key.as_bytes(), value.as_bytes()) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::Sled(error)),
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        match self.db.remove(key.as_bytes()) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::Sled(error)),
        }
    }
}

// trait ThreadPool {
//     /// Creates a new thread pool, immediately spawning the specified number of threads.
//     ///
//     /// Returns an error if any thread fails to spawn. All previously-spawned threads are terminated.
//     fn new(threads: u32) -> Result<dyn ThreadPool> where Self: Sized;
//     /// Spawn a function into the threadpool.
//     ///
//     /// Spawning always succeeds, but if the function panics the threadpool continues
//     /// to operate with the same number of threads — the thread count is not reduced
//     /// nor is the thread pool destroyed, corrupted or invalidated.
//     fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static;
// }