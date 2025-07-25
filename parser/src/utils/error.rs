use std::{
    io::Error,
    string::{FromUtf8Error, FromUtf16Error},
};

#[derive(Debug)]
pub enum SeekKind {
    SeekTo,
    Pad,
}

/// Errors that come from binary read / write operations.
// TODO(nenikitov): Add position to all error variants
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
    /// An error occurred when parsing UTF-8 strings.
    FromUtf8(FromUtf8Error),
    /// An error occurred when parsing UTF-16 strings.
    FromUtf16(FromUtf16Error),
}

impl From<Error> for BinErrorKind {
    fn from(value: Error) -> Self {
        Self::Io(value)
    }
}

impl From<FromUtf8Error> for BinErrorKind {
    fn from(value: FromUtf8Error) -> Self {
        Self::FromUtf8(value)
    }
}

impl From<FromUtf16Error> for BinErrorKind {
    fn from(value: FromUtf16Error) -> Self {
        Self::FromUtf16(value)
    }
}

/// Result that comes from binary read / write operations.
pub type BinResult<T> = Result<T, BinErrorKind>;
