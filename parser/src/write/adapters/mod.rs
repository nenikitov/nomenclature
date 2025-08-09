mod sealed {
    pub trait BinWriteExt<Writer, Args> {}
}

use std::io::{Seek, SeekFrom, Write};

use crate::prelude::*;

pub trait BinWriteExt<Writer, Args>: sealed::BinWriteExt<Writer, Args>
where
    Self: Sized + BinWrite<Writer, Args>,
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

impl<T, Writer, Args> sealed::BinWriteExt<Writer, Args> for T
where
    T: BinWrite<Writer, Args>,
    Writer: Write + Seek,
{
}

impl<T, Writer, Args> BinWriteExt<Writer, Args> for T
where
    T: BinWrite<Writer, Args> + sealed::BinWriteExt<Writer, Args>,
    Writer: Write + Seek,
{
}
