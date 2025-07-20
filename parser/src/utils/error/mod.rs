use std::io::Error;

#[allow(unused_imports)] // Is used in the doc
use crate::prelude::*;

/// Errors that come from binary read / write operations.
#[derive(Debug)]
pub enum BinErrorKind {
    /// An assertion failed (from [`BinReadExt::assert`] function).
    AssertionFailed { pos: u64, message: String },
    /// An error occurred in the stream while reading / writing / seeking the data.
    Io(Error),
}

impl From<Error> for BinErrorKind {
    fn from(value: Error) -> Self {
        Self::Io(value)
    }
}

/// Result that comes from binary read / write operations.
pub type BinResult<T> = Result<T, BinErrorKind>;
