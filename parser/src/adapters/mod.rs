mod sealed {
    pub trait BinReadExt<Reader, Args, Out> {}
}

use std::io::{Read, Seek, SeekFrom};

use crate::prelude::*;

/// Allows chaining adapters to create more complex parsers.
///
/// This trait cannot be implemented.
///
/// # Implementing custom extension methods
///
/// Even though this can be achievend through `.repeat`, here is an example of a custom adapter.
///
/// ```
/// use parser::prelude::*;
/// use std::io::{Read, Seek, Cursor};
///
/// trait MyExtension<Reader, Args, Out>
/// where
///     Self: Sized + BinReadCollect<Reader, Args, Out>,
///     Reader: Read + Seek,
/// {
///     fn parse_3(self) -> impl BinReadCollect<Reader, Args, [Out; 3]>
///     where
///         Self: Clone, // Needed to repeat parsing multiple times
///         Args: Clone, // Needed to repeat parsing multiple times
///     {
///         move |reader: &mut Reader, endian: Endian, args: Args| {
///             let first = self.clone().collect(reader, endian, args.clone())?;
///             let second = self.clone().collect(reader, endian, args.clone())?;
///             let third = self.clone().collect(reader, endian, args.clone())?;
///             Ok([first, second, third])
///         }
///     }
/// }
///
/// impl<Reader, Args, Out> MyExtension<Reader, Args, Out> for BinReadCollect<Reader, Args, Out>
/// where
///     Reader: Read + Seek
/// {}
///
/// let data = Cursor::new(vec![10, 20, 30]);
/// assert_eq!(
///     u8::reader().parse_3().collect(&mut data, Endian::Little, ()),
///     Ok([10, 20, 30])
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
    /// If reading fails, a [`BinError`] variant is returned.
    /// The stream is returned to the position before an error.
    ///
    /// # Arguments
    ///
    /// * `reader`: Stream from which to read.
    /// * `endian`: Target endianness.
    /// * `args`: Arguments required for parsing.
    fn collect(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out> {
        let pos = reader.stream_position()?;
        match self.collect_non_backtracking(reader, endian, args, BinReadCollectToken(())) {
            Err(e) => {
                reader.seek(SeekFrom::Start(pos))?;
                Err(e)
            }
            Ok(v) => Ok(v),
        }
    }

    /// Make an assertion about the value that is read.
    ///
    /// * `assertion`: Function that should return `true` if the value is valid.
    /// * `message`: Function that should return an error message explaining the assertion.
    fn assert<C, E>(self, assertion: C, message: E) -> impl BinReadCollect<Reader, Args, Out>
    where
        C: Fn(&Out) -> bool,
        E: Fn(&Out) -> String,
    {
        move |reader: &mut Reader, endian, args| {
            let pos = reader.stream_position()?;
            let value = self.collect(reader, endian, args)?;
            if !assertion(&value) {
                Err(BinError::AssertionFailed {
                    pos,
                    message: message(&value),
                })
            } else {
                Ok(value)
            }
        }
    }

    /// Map a value being read from one type to another.
    ///
    /// * `map`: Function that is used for conversion.
    fn map<C, Out2>(self, map: C) -> impl BinReadCollect<Reader, Args, Out2>
    where
        C: Fn(Out) -> Out2,
    {
        move |reader: &mut Reader, endian, args| self.collect(reader, endian, args).map(map)
    }

    fn repeat(self, count: usize) -> impl BinReadCollect<Reader, Args, Vec<Out>>
    where
        Self: Clone,
        Args: Clone,
    {
        move |reader: &mut Reader, endian, args: Args| {
            (0..count)
                .map(|_| self.clone().collect(reader, endian, args.clone()))
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
