use std::{
    io::{Read, Seek},
    marker::PhantomData,
    num::NonZero,
};

use crate::prelude::*;

// BinReadCollect

impl<F, Reader, Args, Out> BinReadCollect<Reader, Args, Out> for F
where
    F: FnMut(&mut Reader, Endian, Args) -> BinResult<Out>,
    Reader: Read + Seek,
{
    fn collect_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadCollectToken,
    ) -> BinResult<Out> {
        self(reader, endian, args)
    }
}

// BinReader

impl<T> BinReader for PhantomData<T> {
    type Args = ();
    type Out = Self;

    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek,
    {
        move |_: &mut Reader, _, _| Ok(PhantomData)
    }
}

macro_rules! impl_binread_wrapped {
    ($($type:ident),* $(,)*) => {
        $(
            impl<T> BinReader for $type::<T>
            where
                T: BinReader<Out = T>,
            {
                type Args = T::Args;
                type Out = Self;

                fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    move |reader: &mut Reader, endian, args| {
                        T::reader().map(Self::new).collect(reader, endian, args)
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
                type Args = ();
                type Out = Self;

                fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    move |reader: &mut Reader, endian, _| {
                        let mut buf = [0; size_of::<$type>()];
                        reader.read_exact(&mut buf)?;
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
                type Args = ();
                type Out = Self;
                fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
                where
                    Reader: Read + Seek,
                {
                    move |reader: &mut Reader, endian, args| {
                        <$type>::reader()
                            .assert(
                                |v| *v != 0,
                                |_| "non-zero value expected, but read a 0".to_string(),
                            )
                            .map(|v| {
                                NonZero::<$type>::new(v)
                                    .expect("we already checked for a non-zero value")
                            })
                            .collect(reader, endian, args)
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
    type Args = ();
    type Out = Self;

    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek,
    {
        move |_: &mut Reader, _, _| Ok(())
    }
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(12)]
    #[tuples::member_name(T)]
    impl<Args> BinReader for #Tuple
    where
        Args: Clone,
        #(#T: BinReader<Args = Args, Out = #T>),*
    {
        type Args = Args;
        type Out = Self;

        fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
        where
            Reader: Read + Seek,
        {
            move |reader: &mut Reader, endian, args: Self::Args| {
                #(let #T = #T::reader().collect(reader, endian, args.clone())?;)*
                Ok(#Tuple)
            }
        }
    }
}
