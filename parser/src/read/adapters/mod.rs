pub mod assert;
pub mod map;
pub mod pad_after;
pub mod pad_before;

mod sealed {
    pub trait BinReadExt<Reader, Args, Out> {}
}

use std::io::{Read, Seek, SeekFrom};

use crate::prelude::*;

// TODO(nenikitov)
// Here are the adapters to write
// - pad_after_to
// - repeat_array
// - repeat_vec
// - restore_position
// - seek_before

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
    fn assert<AssertFn, MessageFn>(
        &mut self,
        assertion: AssertFn,
        message: MessageFn,
    ) -> impl BinReadCollect<Reader, Args, Out>
    where
        AssertFn: Fn(&Out) -> bool,
        MessageFn: Fn(&Out) -> String,
    {
        assert::ReadAssert::new(self, assertion, message)
    }

    /// Map a value being read from one type to another.
    ///
    /// * `map`: Function that is used for conversion.
    fn map<MapFn, Out2>(&mut self, map: MapFn) -> impl BinReadCollect<Reader, Args, Out2>
    where
        MapFn: Fn(Out) -> Out2,
    {
        map::ReadMap::new(self, map)
    }

    /// Skip an amount of bytes after a value.
    ///
    // TODO(nenikitov): But should it fail while reading only?
    /// Will not fail if the stream has ended during padding.
    ///
    /// * `padding`: Number of bytes to pad the value with.
    fn pad_after(&mut self, padding: usize) -> impl BinReadCollect<Reader, Args, Out> {
        pad_after::ReadPadAfter::new(self, padding)
    }

    /// Skip an amount of bytes before a value.
    ///
    /// * `padding`: Number of bytes to pad the value with.
    fn pad_before(&mut self, padding: usize) -> impl BinReadCollect<Reader, Args, Out> {
        pad_before::ReadPadBefore::new(self, padding)
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn collect_backtracks_on_error() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x00, 0xA3, 0x66]);
        // Some padding to check the position of the error too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());

        let result = (|reader: &mut Cursor<_>, _, _| -> BinResult<()> {
            reader.seek_relative(2)?;
            reader.read_exact(&mut [0; 2])?;
            Err(BinErrorKind::AssertionFailed {
                pos: 100,
                message: "whatever".to_string(),
            })
        })
        .collect(&mut data, Endian::Big, ());
        assert_matches!(
            result,
            Err(BinErrorKind::AssertionFailed {
                pos: 100,
                message,
            }) if message == "whatever"
        );
        assert_matches!(data.stream_position(), Ok(1));
    }
}
