//! # KvStore, a CLI Key-Value Store
//!
//! Implementation of command-line redis-like key-value store
//! as part of [PingCAP Talent Plan](https://github.com/pingcap/talent-plan).
//!
//! Nothing fancy, but should allow you to [KvStore::set], [KvStore::get] and [KvStore::remove]
//! in a in-memory cache.
// #![warn(missing_docs)]
mod error;
mod store;
pub mod thread_pool;

pub use error::{Error, Result};
pub use store::{KiwiEngine, KiwiStore, SledStore};
