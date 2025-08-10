pub mod assert;
pub mod map;
pub mod pad_after;
pub mod pad_after_to;
pub mod pad_before;
pub mod repeat_array;
pub mod repeat_vec;
pub mod repeat_vec_args_iter;
pub mod restore_position;
pub mod seek_before;

mod sealed {
    pub trait BinReadExt<Reader, Args, Out> {}
}

use std::{
    fmt::Debug,
    io::{Read, Seek, SeekFrom},
};

use crate::prelude::*;

// TODO(nenikitov)
// Here are the adapters to write
// - with_args

/// Allows chaining adapters to create more complex parsers.
///
/// This trait is sealed and cannot be implemented.
///
/// # Implementing custom extension methods
///
// TODO(nenikitov): add this line - Even though this can be achievend through [`BinReadExt::repeat`], here is an example of a custom adapter.
///
/// ## With a callback
///
/// ```
/// use std::io::{Read, Seek, Cursor};
/// use nomenclature::prelude::*;
///
/// // Put all your extension methods in a trait
/// trait MyExtension<Reader, Args, Out>
/// where
///     Self: Sized + BinRead<Reader, Args, Out>,
///     Reader: Read + Seek,
/// {
///     fn parse_3(&mut self) -> impl BinRead<Reader, Args, [Out; 3]>
///     where
///         Args: Clone, // Needed to repeat parsing multiple times
///     {
///         |reader: &mut Reader, endian: Endian, args: Args| {
///             let first = self.read(reader, endian, args.clone())?;
///             let second = self.read(reader, endian, args.clone())?;
///             let third = self.read(reader, endian, args.clone())?;
///             Ok([first, second, third])
///         }
///     }
/// }
///
/// // Create a blanket implementation for `BinRead`
/// impl<T, Reader, Args, Out> MyExtension<Reader, Args, Out> for T
/// where
///     Reader: Read + Seek,
///     T: BinRead<Reader, Args, Out>
/// {
/// }
///
/// // Now you can use it
/// let mut data = Cursor::new(vec![10, 20, 30]);
/// assert_eq!(
///     u8::reader()
///        .parse_3()
///        .read(&mut data, Endian::Little, ())
///        .unwrap(),
///     [10, 20, 30]
/// );
/// ```
///
/// ## With a struct
///
/// ```
/// use std::io::{Cursor, Read, Seek};
/// use nomenclature::prelude::*;
///
/// pub struct Collect3<'f, F> {
///     f: &'f mut F,
/// }
///
/// impl<'f, F, Reader, Args, Out> BinRead<Reader, Args, [Out; 3]> for Collect3<'f, F>
/// where
///     Reader: Read + Seek,
///     F: BinRead<Reader, Args, Out>,
///     Args: Clone, // Needed to repeat parsing multiple times
/// {
///     fn read_non_backtracking(
///         &mut self,
///         reader: &mut Reader,
///         endian: Endian,
///         args: Args,
///         _: BinReadToken,
///     ) -> BinResult<[Out; 3]> {
///         let first = self.f.read(reader, endian, args.clone())?;
///         let second = self.f.read(reader, endian, args.clone())?;
///         let third = self.f.read(reader, endian, args.clone())?;
///         Ok([first, second, third])
///     }
/// }
///
/// // Put all your extension methods in a trait
/// trait MyExtension<Reader, Args, Out>
/// where
///     Self: Sized + BinRead<Reader, Args, Out>,
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
/// // Create a blanket implementation for `BinRead`
/// impl<T, Reader, Args, Out> MyExtension<Reader, Args, Out> for T
/// where
///     Reader: Read + Seek,
///     T: BinRead<Reader, Args, Out>,
/// {
/// }
///
/// // Now you can use it
/// let mut data = Cursor::new(vec![10, 20, 30]);
/// assert_eq!(
///     u8::reader()
///         .parse_3()
///         .read(&mut data, Endian::Little, ())
///         .unwrap(),
///     [10, 20, 30]
/// );
/// ```
pub trait BinReadExt<Reader, Args, Out>: sealed::BinReadExt<Reader, Args, Out>
where
    Self: Sized + BinRead<Reader, Args, Out>,
    Reader: Read + Seek,
{
    /// Read and construct an `Out` value from the stream, advancing it in the process to after the value.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when reading fails.
    ///
    /// The stream is returned to the position before an error.
    ///
    /// # Arguments
    ///
    /// * `reader`: Stream from which to read.
    /// * `endian`: Target endianness.
    /// * `args`: Arguments required for parsing.
    fn read(&mut self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out> {
        let pos = reader.bin_stream_position()?;
        match self.read_non_backtracking(reader, endian, args, BinReadToken(())) {
            Err(e) => {
                reader.bin_seek(SeekFrom::Start(pos), pos)?;
                Err(e)
            }
            Ok(v) => Ok(v),
        }
    }

    /// Fail if a specified condition on a parsed value doesn't pass.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    /// - [`BinErrorKind::Assertion`] when `assertion` returns `false`.
    ///
    /// # Arguments
    ///
    /// * `assertion`: Function that should return `true` if the parsed value is valid.
    /// * `message`: Function that should return an error message explaining the validation.
    fn assert<AssertFn, MessageFn>(
        &mut self,
        assertion: AssertFn,
        message: MessageFn,
    ) -> impl BinRead<Reader, Args, Out>
    where
        AssertFn: Fn(&Out) -> bool,
        MessageFn: Fn(&Out) -> String,
        Out: Debug,
    {
        assert::Assert::new(self, assertion, message)
    }

    /// Map a value being read from one type to another.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    ///
    /// # Arguments
    ///
    /// * `map`: Function that is used for conversion.
    fn map<MapFn, Out2>(&mut self, map: MapFn) -> impl BinRead<Reader, Args, Out2>
    where
        MapFn: Fn(Out) -> Out2,
    {
        map::Map::new(self, map)
    }

    /// Skip an amount of bytes after a value.
    ///
    // TODO(nenikitov): But should it fail while reading only?
    /// Will not fail if the stream has ended during padding.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    /// - [`BinErrorKind::Seek`] when `padding` value too large to be skipped by for [`Read`].
    ///
    /// # Arguments
    ///
    /// * `padding`: Number of bytes to pad the value with.
    fn pad_after(&mut self, padding: usize) -> impl BinRead<Reader, Args, Out> {
        pad_after::PadAfter::new(self, padding)
    }

    /// Skip some bytes after the value so the stream always advances by a given amount.
    ///
    // TODO(nenikitov): But should it fail while reading only?
    /// Will not fail if the stream has ended during padding.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    /// - [`BinErrorKind::Seek`] when `size` value too large to be skipped by for [`Read`].
    /// - [`BinErrorKind::Size`] when the value is larger than the given `size`.
    ///
    /// # Arguments
    ///
    /// * `size`: Length to which the value must be padded to.
    fn pad_after_to(&mut self, size: usize) -> impl BinRead<Reader, Args, Out> {
        pad_after_to::PadAfterTo::new(self, size)
    }

    /// Skip an amount of bytes before a value.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    /// - [`BinErrorKind::Seek`] when `padding` value too large to be skipped by for [`Read`].
    ///
    /// # Arguments
    ///
    /// * `padding`: Number of bytes to pad the value with.
    fn pad_before(&mut self, padding: usize) -> impl BinRead<Reader, Args, Out> {
        pad_before::PadBefore::new(self, padding)
    }

    /// Repeat the parser an amount of times, collecting the results into an array.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    fn repeat_array<const N: usize>(&mut self) -> impl BinRead<Reader, Args, [Out; N]>
    where
        Args: Clone,
    {
        repeat_array::RepeatArray::new(self)
    }

    /// Repeat the parser an amount of times, collecting the results into a vector.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    ///
    /// # Arguments
    ///
    /// * `len`: Number of values to parse.
    fn repeat_vec(&mut self, len: usize) -> impl BinRead<Reader, Args, Vec<Out>>
    where
        Args: Clone,
    {
        repeat_vec::RepeatVec::new(self, len)
    }

    /// Repeat the parser an amount of times, each time applying different arguments, collecting the results into a vector.
    ///
    /// Changes arguments to be an iterator outputting the arguments for an inner parser.
    /// The produced vector will have the same length as this iterator, so it must be finite.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    fn repeat_vec_args_iter<It>(&mut self) -> impl BinRead<Reader, It, Vec<Out>>
    where
        It: IntoIterator<Item = Args>,
    {
        repeat_vec_args_iter::RepeatVecArgsIter::new(self)
    }

    /// Read the value without advancing the stream.
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    fn restore_position(&mut self) -> impl BinRead<Reader, Args, Out> {
        restore_position::RestorePosition::new(self)
    }

    /// Position the stream before reading the value.
    ///
    /// If succeeds, the stream is kept in the position right after the parsed value.
    /// If you need to restore position, use [`BinReadExt::restore_position`].
    ///
    /// # Errors
    ///
    /// - [`BinErrorKind`] when parsing of the inner value fails.
    /// - [`BinErrorKind::Seek`] when `position` value too large to be set to by for [`Read`].
    ///
    /// # Arguments
    ///
    /// * `position`: Position to which set the stream.
    fn seek_before(&mut self, position: usize) -> impl BinRead<Reader, Args, Out> {
        seek_before::SeekBefore::new(self, position)
    }
}

impl<T, Reader, Args, Out> sealed::BinReadExt<Reader, Args, Out> for T
where
    T: BinRead<Reader, Args, Out>,
    Reader: Read + Seek,
{
}

impl<T, Reader, Args, Out> BinReadExt<Reader, Args, Out> for T
where
    T: BinRead<Reader, Args, Out> + sealed::BinReadExt<Reader, Args, Out>,
    Reader: Read + Seek,
{
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn read_backtracks_on_error() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x00, 0xA3, 0x66]);
        // Some padding to check the position of the error too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        let result = (|reader: &mut Cursor<_>, _, _| -> BinResult<()> {
            reader.seek(SeekFrom::Current(2)).unwrap();
            reader.read_exact(&mut [0; 2]).unwrap();
            Err(BinError::new(
                Some(100),
                BinErrorKind::Assertion {
                    value: "value".to_string(),
                    message: "whatever".to_string(),
                },
            ))
        })
        .read(&mut data, Endian::Big, ());
        assert_eq!(
            result,
            Err(BinError::new(
                Some(100),
                BinErrorKind::Assertion {
                    value: "value".to_string(),
                    message: "whatever".to_string(),
                }
            ))
        );
        assert_eq!(data.bin_stream_position(), Ok(1));
    }
}
