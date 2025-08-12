use std::io::{Seek, Write};

use crate::prelude::Endian;

struct BinWriteToken(());

trait BinWrite<Writer, Args, In>
where
    Writer: Write + Seek,
{
    fn write_non_backtracking(
        &mut self,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
        _: BinWriteToken,
    ) -> Result<(), ()>;

    fn inner(&self) -> &In;
}

trait BinWriter {
    type Args<'a>;
    type In;

    fn writer_mapped<'a, Writer>(
        this: &Self::In,
    ) -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek;
}

trait BinWriterSelf
where
    Self: BinWriter<In = Self> + Sized,
{
    fn writer<'a, Writer>(&self) -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        Self::writer_mapped(self)
    }
}

impl<T> BinWriterSelf for T where T: BinWriter<In = Self> {}

impl<F, Writer, Args, In> BinWrite<Writer, Args, In> for (&In, F)
where
    F: FnMut(&mut Writer, Endian, Args) -> Result<(), ()>,
    Writer: Write + Seek,
{
    fn write_non_backtracking(
        &mut self,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
        _: BinWriteToken,
    ) -> Result<(), ()> {
        (self.1)(writer, endian, args)
    }

    fn inner(&self) -> &In {
        self.0
    }
}

struct Assert<'f, F, Assertion> {
    inner: &'f mut F,
    assertion: Assertion,
}

impl<'f, F, Assertion, Writer, Args, In> BinWrite<Writer, Args, In> for Assert<'f, F, Assertion>
where
    Writer: Write + Seek,
    F: BinWrite<Writer, Args, In>,
    Assertion: Fn(&In) -> bool,
{
    fn write_non_backtracking(
        &mut self,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
        _: BinWriteToken,
    ) -> Result<(), ()> {
        if !(self.assertion)(self.inner.inner()) {
            Err(())
        } else {
            self.inner.write(writer, endian, args)
        }
    }

    fn inner(&self) -> &In {
        self.inner.inner()
    }
}

trait BinWriteExt<Writer, Args, In>
where
    Self: Sized + BinWrite<Writer, Args, In>,
    Writer: Write + Seek,
{
    fn write(&mut self, writer: &mut Writer, endian: Endian, args: Args) -> Result<(), ()> {
        self.write_non_backtracking(writer, endian, args, BinWriteToken(()))
    }

    fn assert<F>(&mut self, assertion: F) -> impl BinWrite<Writer, Args, In>
    where
        F: Fn(&In) -> bool,
    {
        Assert {
            inner: self,
            assertion,
        }
    }
}

impl<T, Writer, Args, In> BinWriteExt<Writer, Args, In> for T
where
    T: BinWrite<Writer, Args, In>,
    Writer: Write + Seek,
{
}

impl BinWriter for u8 {
    type Args<'a> = ();
    type In = Self;

    fn writer_mapped<'a, Writer>(this: &Self::In) -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        (this, |_: &mut Writer, _, _| Ok(()))
    }
}

impl BinWriter for u16 {
    type Args<'a> = ();
    type In = u8;

    fn writer_mapped<'a, Writer>(this: &Self::In) -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        (this, |_: &mut Writer, _, _| Ok(()))
    }
}

impl<T0, T1> BinWriter for (T0, T1)
where
    for<'a> T0::Args<'a>: Clone,
    T0: BinWriter,
    for<'a> T1: BinWriter<Args<'a> = T0::Args<'a>>,
{
    type Args<'a> = T0::Args<'a>;
    type In = (T0::In, T1::In);

    fn writer_mapped<'a, Writer>(this: &Self::In) -> impl BinWrite<Writer, Self::Args<'a>, Self::In>
    where
        Writer: Write + Seek,
    {
        (this, |writer: &mut Writer, endian, args: Self::Args<'a>| {
            T0::writer_mapped(&this.0).write(writer, endian, args.clone())?;
            T1::writer_mapped(&this.1).write(writer, endian, args.clone())?;
            Ok(())
        })
    }
}

fn test() {
    let mut buf = std::io::Cursor::new(vec![30u8]);

    _ = u16::writer_mapped(&10)
        .assert(|&a| a > 10)
        .write(&mut buf, Endian::Little, ());

    _ = <(u16, u16)>::writer_mapped(&(10u8, 20u8)).write(&mut buf, Endian::Little, ());
}
