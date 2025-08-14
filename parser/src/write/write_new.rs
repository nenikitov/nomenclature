use std::{
    io::{Seek, Write},
    marker::PhantomData,
};

use crate::prelude::Endian;

trait BinWrite<Writer, Args, In>
where
    Writer: Write + Seek,
{
    fn write_non_backtracking(
        &mut self,
        value: &In,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
    ) -> Result<(), ()>;
}

impl<F, Writer, Args, In> BinWrite<Writer, Args, In> for F
where
    F: FnMut(&In, &mut Writer, Endian, Args) -> Result<(), ()>,
    Writer: Write + Seek,
{
    fn write_non_backtracking(
        &mut self,
        value: &In,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
    ) -> Result<(), ()> {
        self(value, writer, endian, args)
    }
}

trait BinWriter {
    type Args<'a>;
    type In;

    fn writer<'a, Writer>() -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek;
}

impl BinWriter for u8 {
    type Args<'a> = ();
    type In = Self;

    fn writer<'a, Writer>() -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        |value: &Self::In, writer: &mut Writer, endian, args| {
            writer.write_all(&value.to_le_bytes()).unwrap();
            Ok(())
        }
    }
}

trait BinWriterStart {
    fn writer(&self) -> impl PossiblyBinWriteTrait<&Self> {
        PossiblyBinWrite(self)
    }
}

struct PossiblyBinWrite<T>(T);

trait PossiblyBinWriteTrait<In> {}

impl<In> PossiblyBinWriteTrait<In> for PossiblyBinWrite<In> {}

trait PossiblyBinWriteTraitExt<In>
where
    Self: PossiblyBinWriteTrait<In> + Sized,
{
    fn write<'a, Writer>(
        self,
        writer: &mut Writer,
        endian: Endian,
        args: In::Args<'a>,
    ) -> Result<(), ()>
    where
        In: BinWriter<In = In>,
        Writer: Write + Seek,
    {
        In::writer().write_non_backtracking(&self, writer, endian, args)
    }

    fn map<In2, MapFn>(self, map: MapFn) -> impl PossiblyBinWriteTrait<In2>
    where
        MapFn: Fn(&In) -> In2,
    {
        Map {
            inner: self,
            map,
            _in: PhantomData,
        }
    }

    fn assert<AssertFn>(self, assertion: AssertFn) -> impl PossiblyBinWriteTrait<In>
    where
        AssertFn: Fn(&In) -> bool,
    {
        Assert {
            inner: self,
            assertion,
        }
    }
}

impl<T, In> PossiblyBinWriteTraitExt<In> for T where T: PossiblyBinWriteTrait<In> {}

struct Map<In, Inner, MapFn> {
    inner: Inner,
    map: MapFn,
    _in: PhantomData<In>,
}

impl<In, In2, Inner, MapFn> PossiblyBinWriteTrait<In2> for Map<In, Inner, MapFn>
where
    Inner: PossiblyBinWriteTrait<In>,
    MapFn: Fn(&In) -> In2,
{
}

impl<Inner, MapFn, Writer, Args, In, In2> BinWrite<Writer, Args, In> for Map<In, Inner, MapFn>
where
    Writer: Write + Seek,
    Inner: BinWrite<Writer, Args, In2>,
    MapFn: Fn(&In) -> In2,
{
    fn write_non_backtracking(
        &mut self,
        value: &In,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
    ) -> Result<(), ()> {
        self.inner
            .write_non_backtracking(&(self.map)(value), writer, endian, args)
    }
}

struct Assert<Inner, AssertFn> {
    inner: Inner,
    assertion: AssertFn,
}

impl<In, Inner, AssertFn> PossiblyBinWriteTrait<In> for Assert<Inner, AssertFn> where
    Inner: PossiblyBinWriteTrait<In>
{
}

impl<Inner, AssertFn, Writer, Args, In> BinWrite<Writer, Args, In> for Assert<Inner, AssertFn>
where
    Writer: Write + Seek,
    Inner: BinWrite<Writer, Args, In>,
    AssertFn: Fn(&In) -> bool,
{
    fn write_non_backtracking(
        &mut self,
        value: &In,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
    ) -> Result<(), ()> {
        if !(self.assertion)(value) {
            Err(())
        } else {
            self.inner
                .write_non_backtracking(value, writer, endian, args)
        }
    }
}

impl<T> BinWriterStart for T {}

fn test() {
    let mut buf = std::io::Cursor::new(vec![10u8]);
    _ = 10u8
        .writer()
        .map(|v| **v as usize)
        .map(|v| *v as u32)
        .assert(|a| true)
        .map(|v| *v as u8)
        .write(&mut buf, Endian::Little, ());
}
