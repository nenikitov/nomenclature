use std::io::{Seek, Write};

use crate::prelude::*;

pub struct BinWriteToken(pub(crate) ());

pub trait BinWrite<Writer, Args>
where
    Writer: Write + Seek,
{
    fn write_non_backtracking(
        &mut self,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
        _: BinWriteToken,
    ) -> BinResult<()>;
}

pub trait BinWriter {
    type Args<'a>;

    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
    where
        Writer: Write + Seek;
}

impl<F, Writer, Args> BinWrite<Writer, Args> for F
where
    F: FnMut(&mut Writer, Endian, Args) -> BinResult<()>,
    Writer: Write + Seek,
{
    fn write_non_backtracking(
        &mut self,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
        _: BinWriteToken,
    ) -> BinResult<()> {
        self(writer, endian, args)
    }
}

impl BinWriter for u8 {
    type Args<'a> = ();

    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
    where
        Writer: Write + Seek,
    {
        |writer: &mut Writer, endian, _| {
            let pos = writer.bin_stream_position()?;
            let buf = match endian {
                Endian::Big => self.to_be_bytes(),
                Endian::Little => self.to_le_bytes(),
            };
            writer.write_all(&buf).map_err(BinError::builder(Some(pos)))
        }
    }
}

fn test() {
    let mut buf = std::io::Cursor::new(vec![]);
    10u8.writer().write_non_backtracking(&mut buf, Endian::Little, (), BinWriteToken(()));
}
