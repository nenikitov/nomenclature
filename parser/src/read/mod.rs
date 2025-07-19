mod impls;

use std::io::{Read, Seek, SeekFrom};

use crate::prelude::*;

pub(super) mod sealed {
    pub trait BinReadCombinator<Reader, Args, Out> {}
    pub struct Token;
}

/// Unit struct used to seal methods.
/// Don't worry about it :)
pub struct BinReadToken(sealed::Token);

/// This trait allows reading data from streams and constructing an `Out` value, while keeping the state in `self`.
/// Because of this sate, an object implementing [`BinReadCollect`] must be instantiated beforehand.
///
/// It should not be directly implemented on parseable types like `u8`, it is used to implement adapters that can be chained.
/// If you are making a parseable type, you should implement [`BinRead`] trait instead.
///
/// Think of this one like [`Iterator`] (has an internal state), while [`BinRead`] is like an [`IntoIterator`] (has no internal state, is an entry point to chaining adapters).
pub trait BinReadCollect<Reader, Args, Out>: sealed::BinReadCombinator<Reader, Args, Out>
where
    Self: Sized,
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
    /// Call a [`BinReadCollect::collect`] wrapper instead.
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
    ///
    /// <div class="warning">
    ///
    /// You don't have to write any backtracking on error code yourself, it is handled for you through [`BinReadCollect::collect`] wrapper.
    ///
    /// </div>
    fn collect_non_backtracking(
        self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadToken,
    ) -> BinResult<Out>;

    /// Read and construct an `Out` value from the stream, advancing it in the process to after the value.
    ///
    /// # Errors
    ///
    /// If reading fails, a [`BinError`] variant is returned.
    /// The stream is returned to the position before an error.
    ///
    /// # Arguments
    ///
    /// * `reader`: Stream from which to read.
    /// * `endian`: Target endianness.
    /// * `args`: Arguments required for parsing.
    ///
    /// # Implementing
    ///
    /// <div class="warning">
    ///
    /// Be careful overwriting this function.
    /// By default, it is a wrapper around [`BinReadCollect::collect_non_backtracking`] that backtracks on error, and this implementation should be good enough for most use cases.
    ///
    /// </div>
    fn collect(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out> {
        let pos = reader.stream_position()?;
        match self.collect_non_backtracking(reader, endian, args, BinReadToken(sealed::Token)) {
            Err(e) => {
                reader.seek(SeekFrom::Start(pos))?;
                Err(e)
            }
            Ok(v) => Ok(v),
        }
    }
}

/// This trait is an entry point to parseable types.
///
/// You should implement this trait for you parseable types.
///
/// Think of this one like [`IntoIterator`] (has no internal state, is an entry point to chaining adapters), while [`BinReadCollect`] is like an [`Iterator`].
pub trait BinRead {
    // TODO(nenikitov): Make this `()` when `associated_type_defaults` gets stabilized
    type Args;

    // TODO(nenikitov): Make this `Self` when `associated_type_defaults` gets stabilized
    type Out;

    /// # Example
    ///
    /// <div class="warning">
    ///     Type hint <code>&mut Reader</code> may be necessary because type inference is wonky with functions.
    /// </div>
    ///
    /// ```
    /// use parser::prelude::*;
    ///
    /// struct MyStruct(u8);
    ///
    /// impl BinRead for MyStruct {
    ///     type Args = u8;
    ///     type Out = Self;
    ///
    ///     fn read<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    ///     where
    ///         Reader: std::io::Read + std::io::Seek,
    ///     {
    ///         move |reader: &mut Reader, endian, args| {
    ///             let val = u8::read().collect(reader, endian, ())?;
    ///             Ok(Self(val + args))
    ///         }
    ///     }
    /// }
    /// ```
    fn read<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek;
}
