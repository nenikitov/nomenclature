pub mod adapters;
pub(crate) mod impls;

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
