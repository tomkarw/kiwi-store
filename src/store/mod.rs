mod kiwi_store;
mod sled_store;

use crate::Result;
use serde::{Deserialize, Serialize};

pub use self::kiwi_store::KiwiStore;
pub use self::sled_store::SledStore;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Command {
    Set((String, String)),
    Remove(String),
}

/// Provides a generic set of actions extracted from KvStore
pub trait KiwiEngine: Clone + Send + 'static {
    fn set(&self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<Option<String>>;
    fn remove(&self, key: String) -> Result<()>;
}
