pub mod adapters;
mod write_new;
pub(crate) mod impls;

use std::io::{Seek, Write};

use crate::prelude::*;

pub struct BinWriteToken(pub(crate) ());

pub trait BinWrite<Writer, Args, In>
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

    fn inner(&self) -> &In;
}

pub trait BinWriter {
    type Args<'a>;
    type In;

    fn writer_mapped<'a, Writer>(
        this: &Self::In,
    ) -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek;
}
