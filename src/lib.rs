//! # KvStore, a CLI Key-Value Store
//!
//! Implementation of command-line redis-like key-value store
//! as part of [PingCAP Talent Plan](https://github.com/pingcap/talent-plan).
//!
//! Nothing fancy, but should allow you to [KvStore::set], [KvStore::get] and [KvStore::remove]
//! in a in-memory cache.
#![warn(missing_docs)]

use std::path::{PathBuf};
use std::{result, io, fmt, error};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufReader, BufRead, Seek, SeekFrom};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde::export::Formatter;
use std::fmt::Display;

/// Result specific for this crate, for now it's error case is `Box<dyn Error>` but this might change
// TODO(tkarwowski): might use https://docs.rs/fehler/1.0.0/fehler/ instead
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoKey(String),
    Offset(String),
    Io(io::Error),
    InvalidData(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoKey(msg) => write!(f, "{}", msg),
            Error::Offset(msg) => write!(f, "{}", msg),
            Error::Io(msg) => write!(f, "{}", msg),
            Error::InvalidData(msg) => write!(f, "{}", msg),
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

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set((String, String)),
    Remove(String),
}

/// KvStore is a key-value store allowing you store values in-memory with O(1) lookup time.
/// # Example
/// ```
/// # use std::error::Error;
/// # use tempfile::TempDir;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// # let some_dir = TempDir::new().unwrap();
/// use kvs::KvStore;
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

impl KvStore {
    /// Open KvStore at a specified location.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let full_path: PathBuf = [path.into(), PathBuf::from("kvs.db")]
            .iter()
            .collect();

        let mut store = HashMap::new();
        if full_path.exists() {
            let mut file = File::open(&full_path)?;
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
                    },
                    Command::Remove(key) => {
                        store.remove(&key);
                    },
                };

                buffer.clear();
                current_offset += read_bytes;
            }
        }

        let write_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)?;

        Ok(KvStore { write_log, full_path, store })
    }

    fn value(&self, offset: u64) -> Result<String> {
        let mut file = File::open(&self.full_path)?;

        file.seek(SeekFrom::Start(offset))?;
        let mut reader = BufReader::new(&file);
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        match serde_json::from_str(&buffer)? {
            Command::Remove(_) => panic!("wrong offset"),
            Command::Set((_, value)) => Ok(value),
        }
    }

    /// Set a value. Overrides the value if key is already present
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.store.insert(key.clone(), self.write_log.seek(SeekFrom::End(0))?);
        let command = serde_json::to_string(&Command::Set((key, value))).unwrap();
        self.write_log.write((command + "\n").as_bytes())?;
        Ok(())
    }

    /// Get a value.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.store.get(&key) {
            Some(offset) => Ok(Some(self.value(*offset)?)),
            None => Ok(None),
        }
    }

    /// Remove a value. If value wasn't present, nothing happens.
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.store.get(&key) {
            Some(_) => {
                self.store.remove(&key);
                let command = serde_json::to_string(
                    &Command::Remove(key)
                ).unwrap();
                self.write_log.write((command + "\n").as_bytes())?;
                Ok(())
            },
            None => Err(Error::NoKey(String::from("Key not found"))),
        }
    }
}
