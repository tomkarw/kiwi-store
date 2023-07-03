use crate::store::KiwiEngine;
use crate::{Error, Result};
use sled::Db;
use std::path::PathBuf;
use std::str;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct SledStoreInner {
    db: Db,
}

impl SledStoreInner {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        match self.db.insert(key.as_bytes(), value.as_bytes()) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::Sled(error)),
        }
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        match self.db.get(key.as_bytes()) {
            Ok(result) => match result {
                Some(value) => Ok(Some(str::from_utf8(&value)?.to_owned())),
                None => Ok(None),
            },
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

#[derive(Debug, Clone)]
pub struct SledStore {
    inner: Arc<RwLock<SledStoreInner>>,
}

impl SledStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(SledStore {
            inner: Arc::new(RwLock::new(SledStoreInner {
                db: sled::open(path.into())?,
            })),
        })
    }
}

impl KiwiEngine for SledStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.inner
            .write()
            .expect("error acquiring lock")
            .set(key, value)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        self.inner.read().expect("error acquiring lock").get(key)
    }

    fn remove(&self, key: String) -> Result<()> {
        self.inner
            .write()
            .expect("error acquiring lock")
            .remove(key)
    }
}
