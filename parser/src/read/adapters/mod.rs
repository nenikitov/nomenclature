pub mod assert;

mod sealed {
    pub trait BinReadExt<Reader, Args, Out> {}
}

use std::io::{Read, Seek, SeekFrom};

use crate::prelude::*;

// TODO(nenikitov)
// Here are the adapters to write
// - map
// - mark_metadata
// - mark_position
// - mark_size
// - pad_after
// - pad_after_to
// - pad_before
// - restore_position
// - seek_before

/// Allows chaining adapters to create more complex parsers.
///
/// This trait is sealed and cannot be implemented.
///
/// # Implementing custom extension methods
///
/// Even though this can be achievend through [`BinReadExt::repeat`], here is an example of a custom adapter.
///
/// ## With a callback
///
/// ```
/// use std::io::{Read, Seek, Cursor};
/// use parser::prelude::*;
///
/// // Put all your extension methods in a trait
/// trait MyExtension<Reader, Args, Out>
/// where
///     Self: Sized + BinReadCollect<Reader, Args, Out>,
///     Reader: Read + Seek,
/// {
///     fn parse_3(&mut self) -> impl BinReadCollect<Reader, Args, [Out; 3]>
///     where
///         Args: Clone, // Needed to repeat parsing multiple times
///     {
///         move |reader: &mut Reader, endian: Endian, args: Args| {
///             let first = self.collect(reader, endian, args.clone())?;
///             let second = self.collect(reader, endian, args.clone())?;
///             let third = self.collect(reader, endian, args.clone())?;
///             Ok([first, second, third])
///         }
///     }
/// }
///
/// // Create a blanket implementation for `BinReadCollect`
/// impl<T, Reader, Args, Out> MyExtension<Reader, Args, Out> for T
/// where
///     Reader: Read + Seek,
///     T: BinReadCollect<Reader, Args, Out>
/// {
/// }
///
/// // Now you can use it
/// let mut data = Cursor::new(vec![10, 20, 30]);
/// assert_eq!(
///     u8::reader()
///        .parse_3()
///        .collect(&mut data, Endian::Little, ())
///        .unwrap(),
///     [10, 20, 30]
/// );
/// ```
///
/// ## With a struct
///
/// ```
/// use std::io::{Cursor, Read, Seek};
/// use parser::prelude::*;
///
/// pub struct Collect3<'f, F> {
///     f: &'f mut F,
/// }
///
/// impl<'f, F, Reader, Args, Out> BinReadCollect<Reader, Args, [Out; 3]> for Collect3<'f, F>
/// where
///     Reader: Read + Seek,
///     F: BinReadCollect<Reader, Args, Out>,
///     Args: Clone, // Needed to repeat parsing multiple times
/// {
///     fn collect_non_backtracking(
///         &mut self,
///         reader: &mut Reader,
///         endian: Endian,
///         args: Args,
///         _: BinReadCollectToken,
///     ) -> BinResult<[Out; 3]> {
///         let first = (self.f).collect(reader, endian, args.clone())?;
///         let second = (self.f).collect(reader, endian, args.clone())?;
///         let third = (self.f).collect(reader, endian, args.clone())?;
///         Ok([first, second, third])
///     }
/// }
///
/// // Put all your extension methods in a trait
/// trait MyExtension<Reader, Args, Out>
/// where
///     Self: Sized + BinReadCollect<Reader, Args, Out>,
///     Reader: Read + Seek,
/// {
///     fn parse_3(&mut self) -> Collect3<'_, Self>
///     where
///         Args: Clone, // Needed to repeat parsing multiple times
///     {
///         Collect3 { f: self }
///     }
/// }
///
/// // Create a blanket implementation for `BinReadCollect`
/// impl<T, Reader, Args, Out> MyExtension<Reader, Args, Out> for T
/// where
///     Reader: Read + Seek,
///     T: BinReadCollect<Reader, Args, Out>,
/// {
/// }
///
/// // Now you can use it
/// let mut data = Cursor::new(vec![10, 20, 30]);
/// assert_eq!(
///     u8::reader()
///         .parse_3()
///         .collect(&mut data, Endian::Little, ())
///         .unwrap(),
///     [10, 20, 30]
/// );
/// ```
pub trait BinReadExt<Reader, Args, Out>: sealed::BinReadExt<Reader, Args, Out>
where
    Self: Sized + BinReadCollect<Reader, Args, Out>,
    Reader: Read + Seek,
{
    /// Read and construct an `Out` value from the stream, advancing it in the process to after the value.
    ///
    /// # Errors
    ///
    /// If reading fails, a [`BinErrorKind`] variant is returned.
    /// The stream is returned to the position before an error.
    ///
    /// # Arguments
    ///
    /// * `reader`: Stream from which to read.
    /// * `endian`: Target endianness.
    /// * `args`: Arguments required for parsing.
    fn collect(&mut self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out> {
        let pos = reader.stream_position()?;
        match self.collect_non_backtracking(reader, endian, args, BinReadCollectToken(())) {
            Err(e) => {
                reader.seek(SeekFrom::Start(pos))?;
                Err(e)
            }
            Ok(v) => Ok(v),
        }
    }

    /// A parser which fails if a specified condition on a parsed value doesn't pass.
    ///
    /// * `assertion`: Function that should return `true` if the parsed value is valid.
    /// * `message`: Function that should return an error message explaining the validation.
    fn assert<AssertionFn, MessageFn>(
        &mut self,
        assertion: AssertionFn,
        message: MessageFn,
    ) -> impl BinReadCollect<Reader, Args, Out>
    where
        AssertionFn: Fn(&Out) -> bool,
        MessageFn: Fn(&Out) -> String,
    {
        assert::ReadAssert {
            f: self,
            assertion,
            message,
        }
    }

    /// Map a value being read from one type to another.
    ///
    /// * `map`: Function that is used for conversion.
    fn map<MapFn, Out2>(&mut self, map: MapFn) -> impl BinReadCollect<Reader, Args, Out2>
    where
        MapFn: Fn(Out) -> Out2,
    {
        move |reader: &mut Reader, endian, args| self.collect(reader, endian, args).map(&map)
    }

    /// Repeat a parser multiple times, collecting it into a [`Vec`].
    ///
    /// * `count`: Number of times to repeat.
    fn repeat(&mut self, count: usize) -> impl BinReadCollect<Reader, Args, Vec<Out>>
    where
        Args: Clone,
    {
        move |reader: &mut Reader, endian, args: Args| {
            (0..count)
                .map(|_| self.collect(reader, endian, args.clone()))
                .collect()
        }
    }
}

impl<T, Reader, Args, Out> sealed::BinReadExt<Reader, Args, Out> for T
where
    T: BinReadCollect<Reader, Args, Out>,
    Reader: Read + Seek,
{
}

impl<T, Reader, Args, Out> BinReadExt<Reader, Args, Out> for T
where
    T: BinReadCollect<Reader, Args, Out> + sealed::BinReadExt<Reader, Args, Out>,
    Reader: Read + Seek,
{
}
