use std::{
    io::{Read, Seek},
    marker::PhantomData,
    num::NonZero,
};

use seq_macro::seq;

use crate::prelude::*;

// BinRead

impl<F, Reader, Args, Out> BinRead<Reader, Args, Out> for F
where
    F: FnMut(&mut Reader, Endian, Args) -> BinResult<Out>,
    Reader: Read + Seek,
{
    fn read_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadToken,
    ) -> BinResult<Out> {
        self(reader, endian, args)
    }
}

// BinReader

impl<T> BinReader for PhantomData<T> {
    type Args<'a> = ();
    type Out = Self;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |_: &mut Reader, _, _| Ok(PhantomData)
    }
}

macro_rules! impl_binread_wrapped {
    ($($type:ident),* $(,)*) => {
        $(
            impl<T> BinReader for $type::<T>
            where
                T: BinReader<Out = T>,
            {
                type Args<'a> = T::Args<'a>;
                type Out = Self;

                fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    |reader: &mut Reader, endian, args| {
                        T::reader().map(Self::new).read(reader, endian, args)
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_binread_wrapped!(
    Box,
);

macro_rules! impl_binread_numeric {
    ($($type:ty),* $(,)*) => {
        $(
            impl BinReader for $type {
                type Args<'a> = ();
                type Out = Self;

                fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    |reader: &mut Reader, endian, _| {
                        let pos = reader.bin_stream_position()?;
                        let mut buf = [0; size_of::<$type>()];
                        reader.read_exact(&mut buf).map_err(BinError::builder(Some(pos)))?;
                        Ok(match endian {
                            Endian::Big => <$type>::from_be_bytes(buf),
                            Endian::Little => <$type>::from_le_bytes(buf),
                        })
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_binread_numeric!(
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64,
);

macro_rules! impl_binread_non_zero {
    ($($type:ty),* $(,)*) => {
        $(
            impl BinReader for NonZero<$type> {
                type Args<'a> = ();
                type Out = Self;

                fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    |reader: &mut Reader, endian, args| {
                        <$type>::reader()
                            .assert(
                                |v| *v != 0,
                                |_| "non-zero value expected, but read a 0".to_string(),
                            )
                            .map(|v| {
                                NonZero::<$type>::new(v)
                                    .expect("we already checked for a non-zero value")
                            })
                            .read(reader, endian, args)
                    }
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

impl BinReader for () {
    type Args<'a> = ();
    type Out = Self;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |_: &mut Reader, _, _| Ok(())
    }
}

macro_rules! impl_binread_tuple {
    ($length:literal) => {
        seq!(I in 1..$length {
            impl<T0, #(T~I,)*> BinReader for (T0, #(T~I,)*)
            where
                T0: BinReader<Out = T0>,
                for<'a> T0::Args<'a>: Clone,
                #(for<'a> T~I: BinReader<Args<'a> = T0::Args<'a>, Out = T~I>,)*
            {
                type Args<'a> = T0::Args<'a>;
                type Out = Self;

                fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    |reader: &mut Reader, endian, args: Self::Args<'a>| {
                        let t0 = T0::reader().read(reader, endian, args.clone())?;
                        #(let t~I = T~I::reader().read(reader, endian, args.clone())?;)*
                        Ok((t0, #(t~I,)*))
                    }
                }
            }
        });
    }
}

// We don't need `Args` to be `Clone` if the tuple is the length of 1, so the manual implementation is more generic
impl<T0> BinReader for (T0,)
where
    T0: BinReader<Out = T0>,
{
    type Args<'a> = T0::Args<'a>;
    type Out = Self;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, args| T0::reader().map(|x| (x,)).read(reader, endian, args)
    }
}

seq!(LENGTH in 2..=12 {
    impl_binread_tuple!(LENGTH);
});

impl<T, const N: usize> BinReader for [T; N]
where
    T: BinReader<Out = T>,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;
    type Out = Self;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, args| {
            T::reader().repeat_array::<N>().read(reader, endian, args)
        }
    }
}

#[derive(Debug, Clone)]
pub struct VecArgs<InnerArgs> {
    pub len: usize,
    pub inner_args: InnerArgs,
}

impl<T> BinReader for Vec<T>
where
    T: BinReader<Out = T>,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = VecArgs<T::Args<'a>>;
    type Out = Self;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, args: Self::Args<'a>| {
            T::reader()
                .repeat_vec(args.len)
                .read(reader, endian, args.inner_args.clone())
        }
    }
}
