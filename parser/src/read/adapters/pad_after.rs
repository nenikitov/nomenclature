use std::io::{Read, Seek, SeekFrom};

use crate::{prelude::*, utils::error::SeekKind};

/// Skip an amount of bytes after a value.
///
// TODO(nenikitov): But should it fail while reading only?
/// Will not fail if the stream has ended during padding.
pub struct ReadPadAfter<'f, F> {
    pub(super) f: &'f mut F,
    pub(super) padding: usize,
}

impl<'f, F> ReadPadAfter<'f, F> {
    pub fn new(f: &'f mut F, padding: usize) -> Self {
        Self { f, padding }
    }
}

impl<F, Reader, Args, Out> BinReadCollect<Reader, Args, Out> for ReadPadAfter<'_, F>
where
    Reader: Read + Seek,
    F: BinReadCollect<Reader, Args, Out>,
{
    fn collect_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadCollectToken,
    ) -> BinResult<Out> {
        let pos = reader.stream_position()?;
        let padding = self
            .padding
            .try_into()
            .map_err(|_| BinErrorKind::InvalidSeek {
                pos,
                kind: SeekKind::Pad,
                value: self.padding,
            })?;

        let value = (self.f).collect(reader, endian, args)?;
        reader.seek(SeekFrom::Current(padding))?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses_value() {
        let mut data = Cursor::new(vec![0x00, 0x52, 0x4F, 0xEE, 0x85, 0x00, 0x00, 0x00]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());
        let result = u32::reader()
            .pad_after(3)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(0x524FEE85))
    }

    #[test]
    fn pads_for_the_next_value() {
        let mut data = Cursor::new(vec![
            0x00, 0x52, 0x4F, 0xEE, 0x85, 0x00, 0x00, 0x00, 0x71, 0x45,
        ]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());
        let _ = u32::reader()
            .pad_after(3)
            .collect(&mut data, Endian::Big, ());
        let result = u16::reader().collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(0x7145))
    }

    #[test]
    fn parses_even_padding_too_big() {
        let mut data = Cursor::new(vec![0x06]);
        let result = u8::reader()
            .pad_after(300)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(_))
    }
}
