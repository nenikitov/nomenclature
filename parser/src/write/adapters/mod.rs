mod sealed {
    pub trait BinWriterSelf {}
    pub trait BinWriteExt<Writer, Args, In> {}
}

use std::io::{Seek, SeekFrom, Write};

use crate::prelude::*;

pub trait BinWriterSelf: sealed::BinWriterSelf
where
    Self: Sized + BinWriter<In = Self>,
{
    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        Self::writer_mapped(self)
    }
}

impl<T> sealed::BinWriterSelf for T where T: BinWriter {}

impl<T> BinWriterSelf for T where T: BinWriter<In = Self> + sealed::BinWriterSelf {}

pub trait BinWriteExt<Writer, Args, In>: sealed::BinWriteExt<Writer, Args, In>
where
    Self: Sized + BinWrite<Writer, Args, In>,
    Writer: Write + Seek,
{
    fn write(&mut self, writer: &mut Writer, endian: Endian, args: Args) -> BinResult<()> {
        let pos = writer.bin_stream_position()?;
        match self.write_non_backtracking(writer, endian, args, BinWriteToken(())) {
            Err(e) => {
                writer.bin_seek(SeekFrom::Start(pos), pos)?;
                Err(e)
            }
            Ok(v) => Ok(v),
        }
    }
}

impl<T, Writer, Args, In> sealed::BinWriteExt<Writer, Args, In> for T
where
    T: BinWrite<Writer, Args, In>,
    Writer: Write + Seek,
{
}

impl<T, Writer, Args, In> BinWriteExt<Writer, Args, In> for T
where
    T: BinWrite<Writer, Args, In> + sealed::BinWriteExt<Writer, Args, In>,
    Writer: Write + Seek,
{
}
