use std::{
    io::{Error, ErrorKind, Seek, SeekFrom},
    string::FromUtf8Error,
};

use widestring::error::Utf16Error;

/// Kind of seeking operation that can be performed.
#[derive(Debug, PartialEq, Eq)]
pub enum SeekKind {
    Seek,
    Size,
    Pad,
}

/// Errors that come from binary read / write operations.
#[derive(Debug, PartialEq, Eq)]
pub enum BinErrorKind {
    /// A failed assertion.
    ///
    /// Can come from:
    /// - [`BinReadExt::assert`](crate::prelude::BinReadExt::assert)
    ///
    /// # Attributes
    ///
    /// * `value` - Representation of an invalid value.
    /// * `message` - Message why this value is invalid.
    Assertion { value: String, message: String },

    /// A value too large to seek by for an underlying stream.
    ///
    /// Can come from:
    /// - [`BinReadExt::pad_after`](crate::prelude::BinReadExt::pad_after)
    /// - [`BinReadExt::pad_after_to`](crate::prelude::BinReadExt::pad_after_to)
    /// - [`BinReadExt::pad_before`](crate::prelude::BinReadExt::pad_before)
    /// - [`BinReadExt::seek_before`](crate::prelude::BinReadExt::seek_before)
    Seek { kind: SeekKind, value: usize },

    /// A value bigger than the size it was expected to be.
    ///
    /// Can come from:
    /// - [`BinReadExt::pad_after_to`](crate::prelude::BinReadExt::pad_after_to)
    Size { expected: usize, got: usize },

    /// An IO error occurred in the stream while reading / writing / seeking the data.
    Io(ErrorKind),

    /// An invalid UTF-8 / UTF-16 string attenmpted to be parsed.
    ///
    /// Can come from:
    /// - [`NullStringUtf8`](crate::prelude::NullStringUtf8)
    /// - [`NullStringUtf16`](crate::prelude::NullStringUtf16)
    StringParsing {
        buffer: Vec<usize>,
        valid_up_to: Option<usize>,
    },
}

impl From<Error> for BinErrorKind {
    fn from(value: Error) -> Self {
        Self::Io(value.kind())
    }
}

impl From<FromUtf8Error> for BinErrorKind {
    fn from(value: FromUtf8Error) -> Self {
        let valid_up_to = Some(value.utf8_error().valid_up_to());
        Self::StringParsing {
            buffer: value.into_bytes().iter().map(|v| *v as usize).collect(),
            valid_up_to,
        }
    }
}

impl From<Utf16Error> for BinErrorKind {
    fn from(value: Utf16Error) -> Self {
        let valid_up_to = Some(value.index());
        Self::StringParsing {
            buffer: value
                .into_vec()
                .expect("string parsing should always be done from a vector")
                .iter()
                .map(|v| *v as usize)
                .collect(),
            valid_up_to,
        }
    }
}

/// Errors that come from binary read / write operations.
#[derive(Debug, PartialEq, Eq)]
pub struct BinError {
    pos: Option<u64>,
    kind: BinErrorKind,
}

impl BinError {
    #[must_use]
    pub fn new(pos: Option<u64>, kind: BinErrorKind) -> Self {
        Self { pos, kind }
    }

    #[must_use]
    pub fn pos(&self) -> Option<u64> {
        self.pos
    }

    #[must_use]
    pub fn kind(&self) -> &BinErrorKind {
        &self.kind
    }

    pub fn builder<E>(pos: Option<u64>) -> impl Fn(E) -> BinError
    where
        E: Into<BinErrorKind>,
    {
        move |error| Self::new(pos, error.into())
    }
}

/// Helper functions around [`Seek`].
pub trait BinResultSeek
where
    Self: Seek,
{
    /// Equivalent to [`Seek::stream_position`], but converts errors to [`BinError`].
    #[allow(clippy::missing_errors_doc)]
    fn bin_stream_position(&mut self) -> BinResult<u64> {
        self.stream_position().map_err(BinError::builder(None))
    }

    /// Equivalent to [`Seek::seek`], but converts errors to [`BinError`].
    #[allow(clippy::missing_errors_doc)]
    fn bin_seek(&mut self, pos: SeekFrom, pos_error: u64) -> BinResult<u64> {
        self.seek(pos).map_err(BinError::builder(Some(pos_error)))
    }
}

impl<T> BinResultSeek for T where T: Seek {}

/// Result that comes from binary read / write operations.
pub type BinResult<T> = Result<T, BinError>;
