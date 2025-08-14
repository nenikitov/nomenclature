use std::{
    io::{Seek, Write},
    ops::Deref,
};

use crate::prelude::Endian;

trait BinWrite<Writer, Args, In>
where
    Writer: Write + Seek,
{
    fn write_non_back_tracking(
        &mut self,
        writer: &mut Writer,
        value: &In,
        endian: Endian,
        args: Args,
    ) -> Result<(), ()>;
}

impl<F, Writer, Args, In> BinWrite<Writer, Args, In> for F
where
    F: FnMut(&mut Writer, &In, Endian, Args) -> Result<(), ()>,
    Writer: Write + Seek,
{
    fn write_non_back_tracking(
        &mut self,
        writer: &mut Writer,
        value: &In,
        endian: Endian,
        args: Args,
    ) -> Result<(), ()> {
        self(writer, value, endian, args)
    }
}

trait BinWriter {
    type Args<'a>;
    type In;

    fn writer<'a, Writer>() -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek;
}

struct BinWriteProxy<T>(T);

trait Test
where
    Self: Sized,
{
    fn writer(&self) -> BinWriteProxy<&Self> {
        BinWriteProxy(&self)
    }
}

impl<T> BinWriter for BinWriteProxy<T>
where
    T: BinWriter,
{
    type Args<'a> = T::Args<'a>;
    type In = T::In;

    fn writer<'a, Writer>() -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        |writer: &mut Writer, value: &Self::In, endian, args| {
            T::writer().write_non_back_tracking(writer, value, endian, args)
        }
    }
}

impl<T> BinWriteProxy<T> {
    fn write<'a, 'i, Writer>(
        &self,
        writer: &mut Writer,
        endian: Endian,
        args: T::Args<'a>,
    ) -> Result<(), ()>
    where
        T: BinWriter<In = T>,
        Writer: Write + Seek,
    {
        T::writer().write_non_back_tracking(writer, &self.0, endian, args)
    }

    fn map<'a, MapFn, T2>(self, map: MapFn) -> BinWriteProxy<T2>
    where
        MapFn: Fn(T) -> T2,
    {
        BinWriteProxy(map(self.0))
    }

    fn repeat(self, times: usize) -> BinWriteProxy<T> {
        BinWriteProxy(self.0)
    }
}

struct Repeat<F>(F, usize);

impl<F, Writer, Args, In> BinWrite<Writer, Args, In> for Repeat<F>
where
    Writer: Write + Seek,
    Args: Clone,
    F: BinWrite<Writer, Args, In>,
{
    fn write_non_back_tracking(
        &mut self,
        writer: &mut Writer,
        value: &In,
        endian: Endian,
        args: Args,
    ) -> Result<(), ()> {
        for _ in 0..self.1 {
            self.0
                .write_non_back_tracking(writer, value, endian, args.clone())?;
        }

        Ok(())
    }
}

impl BinWriter for u8 {
    type Args<'a> = ();
    type In = Self;

    fn writer<'a, Writer>() -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        |writer: &mut Writer, value: &Self::In, endian, args| todo!()
    }
}

impl<T> Test for T {}

fn test() {
    let mut buf = std::io::Cursor::new(vec![10u8]);
    _ = 10u16
        .writer()
        .map(|v| *v as u8)
        .write(&mut buf, Endian::Big, ());
}
