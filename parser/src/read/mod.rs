pub mod adapters;
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
///
/// # Implementing custom extension methods
///
/// See [this section](BinReadExt#implementing-custom-extension-methods).
pub trait BinReadCollect<Reader, Args, Out>
where
    Reader: Read + Seek,
{
    /// Read and construct an `Out` value from the stream, advancing it in the process to after the value.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when reading fails.
    ///
    /// <div class="warning">
    ///
    /// The reader is not returned to the position before an error.
    /// Because of that, this function should not be called directly (there is a token argument preventing this).
    /// Call a [`BinReadExt::collect`] wrapper instead.
    ///
    /// </div>
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
    ///
    /// # Arguments
    ///
    /// * `reader`: Stream from which to read.
    /// * `endian`: Target endianness.
    /// * `args`: Arguments required for parsing.
    /// * `_`: Token to prevent calling this function directly.
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
///
/// # Implementing custom parseable types
///
// TODO(nenikitov): Add section about derive macros when these will be done
///
/// ## Manually
///
/// ```
/// use std::{io::Cursor, num::NonZero};
/// use parser::prelude::*;
///
/// // Declare your struct
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct MyCustomType {
///     a: u32,
///     b: u8,
///     c: NonZero<i16>,
/// }
///
/// // Implement the reader
/// impl BinReader for MyCustomType {
///     type Args = u32;
///     type Out = Self;
///
///     fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
///     where
///         Reader: std::io::Read + std::io::Seek,
///     {
///         move |reader: &mut Reader, endian, args| {
///             let a = u32::reader()
///                 .map(|v| v + args)
///                 .collect(reader, endian, ())?;
///             let b = u8::reader().collect(reader, endian, ())?;
///             let c = <NonZero<i16>>::reader().collect(reader, endian, ())?;
///             Ok(Self { a, b, c })
///         }
///     }
/// }
///
/// // Now you can use it
/// let mut data = Cursor::new(vec![
///     0x00, 0x00, 0x17, 0x36, // `a`
///     0x93, // `b`
///     0x0F, 0xA7, // `c`
/// ]);
/// assert_eq!(
///     MyCustomType::reader()
///         .collect(&mut data, Endian::Big, 2)
///         .unwrap(),
///     MyCustomType {
///         a: 0x17_38,
///         b: 0x93,
///         c: <NonZero<i16>>::new(0x0FA7).unwrap()
///     }
/// );
/// ```
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
    ///
    /// <div class="warning">
    ///
    /// Most of the times type inference will be wonky if you use a callback directly. If you get
    ///
    /// ```txt
    /// error[E0282]: type annotations needed
    ///   --> your/file.rs:34:25
    ///    |
    /// 34 | move |reader, endian, args| {
    ///    |       ^^^^^^ cannot infer type
    /// ```
    ///
    /// or
    ///
    /// ```txt
    /// error: implementation of `FnOnce` is not general enough
    ///    --> your/file.rs:181:9
    ///     |
    /// 181 | / move |reader, endian, args| {
    /// 182 | | }
    ///     | |_^ implementation of `FnOnce` is not general enough
    /// ```
    ///
    /// You most likely need to explicitly type hint the reader:
    ///
    /// ```
    /// # use parser::prelude::*;
    /// #
    /// # pub struct MyCustomType();
    /// #
    /// # impl BinReader for MyCustomType {
    /// # type Args = ();
    /// # type Out = ();
    /// #
    ///   fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    ///   where
    ///       Reader: std::io::Read + std::io::Seek,
    ///   {
    ///       move |reader: &mut Reader, endian, args| {
    ///           /* Your code here */
    /// #         Ok(())
    ///       }
    ///   }
    /// # }
    /// ```
    ///
    /// </div>
    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek;
}
