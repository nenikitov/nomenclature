use std::io::{Read, Seek};

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
    fn collect_non_backtracking(
        self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadToken,
    ) -> BinResult<Out> {
        self(reader, endian, args)
    }
}

macro_rules! impl_binread_primitive {
    ($($type:ty),* $(,)*) => {
        $(
            impl BinRead for $type {
                type Args = ();
                type Out = Self;

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

macro_rules! impl_binread_non_zero {
    ($($type:ty),* $(,)*) => {
        $(
            impl BinRead for std::num::NonZero<$type> {
                type Args = ();
                type Out = Self;

                fn read<Reader>() -> impl $crate::prelude::BinReadCollect<Reader, Self::Args, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    <$type>::read()
                        .assert(
                            |v| *v != 0,
                            |_| "non-zero value expected, but read a 0".to_string(),
                        )
                        .map(|v| std::num::NonZero::<$type>::new(v).expect("we already checked for a non-zero value"))
                    }
                }
        )*
    };
}

#[rustfmt::skip]
impl_binread_non_zero!(
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
);
