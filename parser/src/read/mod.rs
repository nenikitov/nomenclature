mod impls;

use std::io::{Read, Seek, SeekFrom};

use crate::utils::{endian::Endian, error::BinResult};

pub trait BinReadCombinator<Reader>
where
    Reader: Read + Seek,
{
    type Out;

    fn read_non_backtracking(&self, reader: &mut Reader, endian: Endian) -> BinResult<Self::Out>;

    #[inline]
    fn read(&self, reader: &mut Reader, endian: Endian) -> BinResult<Self::Out> {
        let pos = reader.stream_position()?;

        match self.read_non_backtracking(reader, endian) {
            e if e.is_err() => {
                reader.seek(SeekFrom::Start(pos))?;
                e
            }
            v => v,
        }
    }
}

pub trait BinRead<Reader>
where
    Reader: Read + Seek,
{
    type Out;

    fn reader() -> impl BinReadCombinator<Reader, Out = Self::Out>;
}

impl<Reader> BinRead<Reader> for u16
where
    Reader: Read + Seek,
{
    type Out = Self;

    fn reader() -> impl BinReadCombinator<Reader, Out = Self::Out> {
        move |reader: &mut Reader, endian| {
            Ok(5)
        }
    }
}

impl<T, Reader, Out> BinReadCombinator<Reader> for T
where
    Reader: Read + Seek,
    T: Fn(&mut Reader, Endian) -> BinResult<Out>,
{
    type Out = Out;

    fn read_non_backtracking(&self, reader: &mut Reader, endian: Endian) -> BinResult<Self::Out> {
        self(reader, endian)
    }
}
