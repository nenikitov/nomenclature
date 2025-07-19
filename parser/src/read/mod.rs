mod impls;

use std::io::{Read, Seek};

use crate::prelude::*;

pub(super) mod sealed {
    pub trait BinReadCombinator<Reader, Args, Out> {}
}

pub trait BinReadCollect<Reader, Args, Out>: sealed::BinReadCombinator<Reader, Args, Out>
where
    Self: Sized,
    Reader: Read + Seek,
{
    fn collect(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out>;
}

pub trait BinRead {
    // TODO(nenikitov): Make this `()` when `associated_type_defaults` gets stabilized
    type Args;
    // TODO(nenikitov): Make this `Self` when `associated_type_defaults` gets stabilized
    type Out;

    fn read<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek;
}
