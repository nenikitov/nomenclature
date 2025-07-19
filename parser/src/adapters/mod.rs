use std::io::{Read, Seek};

use crate::prelude::*;

pub trait BinReadAdapter<Reader, Args, Out>: BinReadCollect<Reader, Args, Out>
where
    Reader: Read + Seek,
{
    fn assert<C, E>(self, callback: C, error_message: E) -> impl BinReadAdapter<Reader, Args, Out>
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

    fn map<C, Out2>(self, callback: C) -> impl BinReadAdapter<Reader, Args, Out2>
    where
        C: Fn(Out) -> Out2,
    {
        move |reader: &mut Reader, endian, args| self.collect(reader, endian, args).map(callback)
    }
}

impl<T, Reader, Args, Out> BinReadAdapter<Reader, Args, Out> for T
where
    T: BinReadCollect<Reader, Args, Out>,
    Reader: Read + Seek,
{
}
