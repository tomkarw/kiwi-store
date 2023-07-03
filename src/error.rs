use std::fmt::Display;
use std::net::AddrParseError;
use std::str;
use std::{error, fmt, io, result};

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
    /// Error while acquiring Mutex
    // PoisonError(sync::PoisonError<sync::MutexGuard<'a, KvStoreInner>>),
    /// Error parsing string to socket address
    AddrParseError(AddrParseError),
    /// Error related to gRPC transport layer in Tonic
    TransportError(tonic::transport::Error),
    /// Error instantiating rayon thread pool
    ThreadPoolBuild(rayon::ThreadPoolBuildError),
    /// Any ad hoc error
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoKey(msg) => write!(f, "{}", msg),
            Error::Offset(msg) => write!(f, "{}", msg),
            Error::Io(msg) => write!(f, "{}", msg),
            Error::InvalidData(msg) => write!(f, "{}", msg),
            Error::Utf8Error(msg) => write!(f, "{}", msg),
            Error::Sled(msg) => write!(f, "{}", msg),
            // Error::PoisonError(msg) => write!(f, "{}", msg),
            Error::Other(msg) => write!(f, "{}", msg),
            Error::AddrParseError(msg) => write!(f, "{}", msg),
            Error::TransportError(msg) => write!(f, "{}", msg),
            Error::ThreadPoolBuild(msg) => write!(f, "{}", msg),
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

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Error::AddrParseError(err)
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Error::TransportError(err)
    }
}

impl From<rayon::ThreadPoolBuildError> for Error {
    fn from(err: rayon::ThreadPoolBuildError) -> Self {
        Error::ThreadPoolBuild(err)
    }
}
