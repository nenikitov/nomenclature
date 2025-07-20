mod impls;

use std::io::{Read, Seek};

use crate::prelude::*;

/// Unit struct used to seal methods.
/// Don't worry about it :)
pub struct BinReadCollectToken(pub(crate) ());

/// Allows reading data from streams and constructing an `Out` value, while keeping the state in `self`.
/// Because of this sate, an object implementing [`BinReadCollect`] must be instantiated beforehand.
///
/// It is used for adapters (like `.pad_before`) that can be chained, not be implemented on parseable types like `u8`.
/// If you are making a parseable type, you should implement [`BinReader`] trait instead.
///
/// Think of this one like [`Iterator`] (has an internal state), while [`BinReader`] is like an [`IntoIterator`] (has no internal state, is an entry point to chaining adapters).
pub trait BinReadCollect<Reader, Args, Out>
where
    Reader: Read + Seek,
{
    /// Read and construct an `Out` value from the stream, advancing it in the process to after the value.
    ///
    /// # Errors
    ///
    /// If reading fails, a [`BinError`] variant is returned.
    ///
    /// <div class="warning">
    ///
    /// The reader is not returned to the position before an error.
    /// Because of that, this function should not be called directly (there is a token argument preventing this).
    /// Call a [`BinReadExt::collect`] wrapper instead.
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// * `reader`: Stream from which to read.
    /// * `endian`: Target endianness.
    /// * `args`: Arguments required for parsing.
    /// * `_`: Token to prevent calling this function directly.
    ///
    /// # Implementing
    ///
    /// This is the function you should implement to make a new stateful reader.
    /// Although you implement it, you would call it through a thin wrapper [`BinReadExt::collect`].
    ///
    /// <div class="warning">
    ///
    /// You don't have to write any backtracking on error code yourself, it is handled for you through [`BinReadExt::collect`] wrapper.
    ///
    /// </div>
    fn collect_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadCollectToken,
    ) -> BinResult<Out>;
}

/// Allows to parse types, serves an entry point to parseable types.
///
/// You should implement this trait for you parseable types.
///
/// Think of this one like [`IntoIterator`] (has no internal state, is an entry point to chaining adapters), while [`BinReadCollect`] is like an [`Iterator`].
pub trait BinReader {
    /// Arguments (context) required to parse the `Out` type.
    ///
    /// You'd usually set it to `()`.
    // TODO(nenikitov): Make this `()` by default when `associated_type_defaults` feature gets stabilized.
    type Args;
    /// Output of the parsing.
    ///
    /// You'd usually set it to `Self`.
    // TODO(nenikitov): Make this `Self` by default when `associated_type_defaults` feature gets stabilized.
    type Out;

    /// Instantiate a [`BinReadCollect`] that can read a stream and return an object of an `Out` type.
    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek;
}
