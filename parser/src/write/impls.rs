use std::{
    io::{Seek, Write},
    marker::PhantomData,
    num::NonZero,
    ops::Deref,
    rc::Rc,
    sync::Arc,
};

use seq_macro::seq;

use crate::prelude::*;

// BinWrite

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

// BinWriter

impl<T> BinWriter for PhantomData<T> {
    type Args<'a> = ();

    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
    where
        Writer: Write + Seek,
    {
        |_: &mut Writer, _, _| Ok(())
    }
}

macro_rules! impl_bin_write_wrapped {
    ($($type:ident),* $(,)*) => {
        $(
            impl<T> BinWriter for $type<T>
            where
                T: BinWriter,
            {
                type Args<'a> = T::Args<'a>;

                fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
                where
                    Writer: Write + Seek,
                {
                    |writer: &mut Writer, endian, args| self.deref().writer().write(writer, endian, args)
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_bin_write_wrapped!(
    Box, Rc, Arc
);

macro_rules! impl_bin_write_numeric {
    ($($type:ty),* $(,)*) => {
        $(
            impl BinWriter for $type {
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
        )*
    };
}

#[rustfmt::skip]
impl_bin_write_numeric!(
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64,
);

// TODO(nenikitov): Implement for `NonZero` types

impl BinWriter for () {
    type Args<'a> = ();

    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
    where
        Writer: Write + Seek,
    {
        |_: &mut Writer, _, _| Ok(())
    }
}

macro_rules! impl_bin_write_tuple {
    ($length:literal) => {
        seq!(I in 1..$length {
            impl<T0, #(T~I,)*> BinWriter for (T0, #(T~I,)*)
            where
                T0: BinWriter,
                for<'a> T0::Args<'a>: Clone,
                #(for<'a> T~I: BinWriter<Args<'a> = T0::Args<'a>>,)*
            {
                type Args<'a> = T0::Args<'a>;

                fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
                where
                    Writer: Write + Seek
                {
                    |writer: &mut Writer, endian, args: Self::Args<'a>| {
                        self.0.writer().write(writer, endian, args.clone())?;
                        //#(self.~I.writer().write(writer, endian, args.clone())?;)*
                        Ok(())
                    }
                }
            }
        });
    };
}

// We don't need `Args` to be `Clone` if the tuple is the length of 1, so the manual implementation is more generic
impl<T0> BinWriter for (T0,)
where
    T0: BinWriter,
{
    type Args<'a> = T0::Args<'a>;

    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
    where
        Writer: Write + Seek,
    {
        |writer: &mut Writer, endian, args| self.0.writer().write(writer, endian, args)
    }
}

#[rustfmt::skip]
seq!(LENGTH in 2..=12 {
    impl_bin_write_tuple!(LENGTH);
});

impl<T, const N: usize> BinWriter for [T; N]
where
    T: BinWriter,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
    where
        Writer: Write + Seek,
    {
        |writer: &mut Writer, endian, args: Self::Args<'a>| {
            self.iter()
                .map(|x| x.writer().write(writer, endian, args.clone()))
                .collect()
        }
    }
}

impl<T> BinWriter for Vec<T>
where
    T: BinWriter,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>>
    where
        Writer: Write + Seek,
    {
        |writer: &mut Writer, endian, args: Self::Args<'a>| {
            self.iter()
                .map(|x| x.writer().write(writer, endian, args.clone()))
                .collect()
        }
    }
}
