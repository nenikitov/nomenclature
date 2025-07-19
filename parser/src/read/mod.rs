mod impls;

use std::io::{Read, Seek, SeekFrom};

use crate::utils::{endian::Endian, error::BinResult};

pub trait BinRead<Reader, Args, Out>
where
    Self: Sized,
    Reader: Read + Seek,
{
    fn read(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out>;

    fn map<Out2, F>(self, callback: F) -> impl BinRead<Reader, Args, Out2>
    where
        F: FnOnce(Out) -> Out2,
    {
        move |reader: &mut Reader, endian, args| self.read(reader, endian, args).map(callback)
    }

    fn pad_after(self, padding: i64) -> impl BinRead<Reader, Args, Out> {
        move |reader: &mut Reader, endian, args| {
            let v = self.read(reader, endian, args);
            reader.seek_relative(padding)?;
            v
        }
    }
}

pub trait BinParser {
    type Args;
    type Out;

    fn parser<Reader>() -> impl BinRead<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek;
}

impl BinParser for u8 {
    type Args = ();
    type Out = u8;

    fn parser<Reader>() -> impl BinRead<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek,
    {
        move |reader: &mut Reader, _, _| {
            let mut buf = [0; 1];
            reader.read_exact(&mut buf)?;
            Ok(u8::from_le_bytes(buf))
        }
    }
}

impl<F, Reader, Args, Out> BinRead<Reader, Args, Out> for F
where
    F: FnOnce(&mut Reader, Endian, Args) -> BinResult<Out>,
    Reader: Read + Seek,
{
    fn read(self, reader: &mut Reader, endian: Endian, args: Args) -> BinResult<Out> {
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

#[cfg(test)]
pub mod test {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn it_works() -> BinResult<()> {
        #[rustfmt::skip]
        let mut data = Cursor::new(
            vec![
            // first
            10,
             // padding
            20, 30, 40,
            // second
            50,
        ]);

        let first =
            u8::parser()
                .map(|a| a + 20)
                .pad_after(3)
                .read(&mut data, Endian::Little, ())?;

        let second = u8::parser().read(&mut data, Endian::Little, ())?;

        assert_eq!(first, 30);
        assert_eq!(second, 50);

        Ok(())
    }
}
