mod impls;

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::utils::{endian::Endian, error::BinResult};

pub trait BinRead<Reader, Args, Out>
where
    Self: Sized,
    Reader: Read + Seek,
{
    fn read(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out>;
}

pub trait BinParser {
    type Args;
    type Out;

    fn parser<Reader>() -> impl BinRead<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek;
}

impl BinParser for u16 {
    type Args = ();
    type Out = u16;

    fn parser<Reader>() -> impl BinRead<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek,
    {
        move |r: &mut Reader, e, a| Ok(10)
    }
}

impl<F, Reader, Args, Out> BinRead<Reader, Args, Out> for F
where
    F: Fn(&mut Reader, Endian, Args) -> BinResult<Out>,
    Reader: Read + Seek,
{
    fn read(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out> {
        self(reader, endian, args)
    }
}

fn test() {
    let a = u16::parser().read(&mut Cursor::new(vec![]), Endian::Little, ());
}
