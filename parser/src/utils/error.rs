use std::io::Error;

#[derive(Debug)]
pub enum SeekKind {
    SeekTo,
    Pad,
}

/// Errors that come from binary read / write operations.
#[derive(Debug)]
pub enum BinErrorKind {
    /// An assertion failed.
    ///
    /// Can come from:
    /// - [`BinReadExt::assert`](crate::prelude::BinReadExt::assert)
    AssertionFailed { pos: u64, message: String },
    /// An invalid seek operation was performed.
    ///
    /// Can come from:
    /// - [`BinReadExt::pad_after`](crate::prelude::BinReadExt::pad_after)
    /// - [`BinReadExt::pad_before`](crate::prelude::BinReadExt::pad_before)
    // TODO(nenikitov): Add more seeking functions
    InvalidSeek {
        pos: u64,
        kind: SeekKind,
        value: usize,
    },
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
