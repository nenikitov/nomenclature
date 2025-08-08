use std::{
    cell::Cell,
    io::{Read, Seek},
};

use crate::prelude::*;

/// A wrapper that stores the value along with the position in which it was read / written.
///
/// When reading, both [`Marker::pos`] and [`Marker::value`] will be populated.
///
/// When writing, [`Marker::pos`] will be populated and [`Default::default`] as a placeholder will be written.
// TODO(nenikitov): add this line - You can later write to it by using [`BinReaderExt::mark_metadata`], [`BinReaderExt::mark_position`], or [`BinReaderExt::mark_size`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Marker<T> {
    pos: Cell<u64>,
    value: T,
}

impl<T> Marker<T> {
    /// Position of the [`Marker::value`] in the stream.
    pub fn pos(&self) -> u64 {
        self.pos.get()
    }

    /// Underlying value.
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> BinReader for Marker<T>
where
    T: BinReader,
{
    type Args<'a> = T::Args<'a>;
    type Out = Marker<T::Out>;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, args| {
            let pos = reader.bin_stream_position()?;
            let value = T::reader().read(reader, endian, args)?;

            Ok(Marker {
                pos: Cell::new(pos),
                value,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn stores_value_when_read() {
        let mut data = Cursor::new(vec![0xCE, 0x55]);

        let result = <Marker<u16>>::reader().read(&mut data, Endian::Big, ());
        assert_matches!(
            result,
            Ok(Marker {
                pos: _,
                value: 0xCE55
            })
        );
    }

    #[test]
    fn stores_position_when_read() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0xCF, 0x25]);
        // Some padding to check the position of too
        let _ = u16::reader().read(&mut data, Endian::Big, ());

        let result = <Marker<u16>>::reader().read(&mut data, Endian::Big, ());
        assert_eq!(
            result,
            Ok(Marker {
                pos: Cell::new(2),
                value: 0xCF25
            })
        );
    }
}
