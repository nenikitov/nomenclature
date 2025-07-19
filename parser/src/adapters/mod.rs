mod sealed {
    pub trait BinReadExt<Reader, Args, Out> {}
}

use std::io::{Read, Seek, SeekFrom};

use crate::prelude::*;

pub trait BinReadExt<Reader, Args, Out>
where
    Self: Sized + BinReadCollect<Reader, Args, Out> + sealed::BinReadExt<Reader, Args, Out>,
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

    fn assert<C, E>(self, callback: C, error_message: E) -> impl BinReadCollect<Reader, Args, Out>
    where
        C: Fn(&Out) -> bool,
        E: Fn(&Out) -> String,
    {
        move |reader: &mut Reader, endian, args| {
            let pos = reader.stream_position()?;
            let value = self.collect(reader, endian, args)?;
            if !callback(&value) {
                Err(BinError::AssertionFailed {
                    pos,
                    message: error_message(&value),
                })
            } else {
                Ok(value)
            }
        }
    }

    fn map<C, Out2>(self, callback: C) -> impl BinReadCollect<Reader, Args, Out2>
    where
        C: Fn(Out) -> Out2,
    {
        move |reader: &mut Reader, endian, args| self.collect(reader, endian, args).map(callback)
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
