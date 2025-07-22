use std::cell::Cell;

use crate::prelude::*;

/// A wrapper that stores the value along with the position in which it was read / written.
///
/// When reading, both [`Marker::pos`] and [`Marker::value`] will be populated.
///
/// When writing, [`Marker::pos`] will be populated and [`Default::default`] as a placeholder will be written.
// TODO(nenikitov): add this line - You can later write to it by using [`BinReaderExt::mark_metadata`], [`BinReaderExt::mark_position`], or [`BinReaderExt::mark_size`].
#[derive(Debug, Default)]
pub struct Marker<M> {
    pos: Cell<u64>,
    value: M,
}

impl<M> Marker<M> {
    /// Position of the [`Marker::value`] in the stream.
    pub fn pos(&self) -> u64 {
        self.pos.get()
    }

    /// Underlying value.
    pub fn value(&self) -> &M {
        &self.value
    }
}

impl<M> BinReader for Marker<M>
where
    M: BinReader<Out = M>,
{
    type Args = M::Args;

    type Out = Self;

    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: std::io::Read + std::io::Seek,
    {
        move |reader: &mut Reader, endian, args| {
            let pos = reader.stream_position()?;
            let value = M::reader().collect(reader, endian, args)?;

            Ok(Self {
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
        let result = <Marker<u16>>::reader().collect(&mut data, Endian::Big, ());
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
        let _ = u16::reader().collect(&mut data, Endian::Big, ());

        let result = <Marker<u16>>::reader().collect(&mut data, Endian::Big, ());
        assert_matches!(
            result,
            Ok(Marker {
                pos,
                value: _
            }) if pos == Cell::new(2)
        );
    }
}
