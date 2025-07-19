use std::io::{Read, Seek, SeekFrom};

use super::sealed;
use crate::prelude::*;

impl<F, Reader, Args, Out> sealed::BinReadCombinator<Reader, Args, Out> for F where
    F: FnOnce(&mut Reader, Endian, Args) -> BinResult<Out>
{
}

impl<F, Reader, Args, Out> BinReadCollect<Reader, Args, Out> for F
where
    F: FnOnce(&mut Reader, Endian, Args) -> BinResult<Out>,
    Reader: Read + Seek,
{
    fn collect(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out> {
        let pos = reader.stream_position()?;
        match self(reader, endian, args) {
            Err(e) => {
                reader.seek(SeekFrom::Start(pos))?;
                Err(e)
            }
            Ok(v) => Ok(v),
        }
    }
}

macro_rules! impl_binread_primitive {
    ($($type:ty),* $(,)*) => {
        $(
            impl BinRead for $type {
                type Args = ();
                type Out = $type;

                fn read<Reader>() -> impl $crate::prelude::BinReadCollect<Reader, Self::Args, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    move |reader: &mut Reader, endian, _| {
                        let mut buf = [0; size_of::<$type>()];
                        reader.read_exact(&mut buf)?;
                        Ok(match endian {
                            $crate::prelude::Endian::Big => <$type>::from_be_bytes(buf),
                            $crate::prelude::Endian::Little => <$type>::from_le_bytes(buf),
                        })
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_binread_primitive!(
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64,
);
