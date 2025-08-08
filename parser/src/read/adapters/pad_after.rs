use std::io::{Read, Seek, SeekFrom};

use crate::{prelude::*, utils::error::SeekKind};

/// Skip an amount of bytes after a value.
///
// TODO(nenikitov): But should it fail while reading only?
/// Will not fail if the stream has ended during padding.
pub struct ReadPadAfter<'f, F> {
    f: &'f mut F,
    padding: usize,
}

impl<'f, F> ReadPadAfter<'f, F> {
    pub(super) fn new(f: &'f mut F, padding: usize) -> Self {
        Self { f, padding }
    }
}

impl<F, Reader, Args, Out> BinRead<Reader, Args, Out> for ReadPadAfter<'_, F>
where
    Reader: Read + Seek,
    F: BinRead<Reader, Args, Out>,
{
    fn read_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadToken,
    ) -> BinResult<Out> {
        let pos = reader.bin_stream_position()?;
        let padding = self.padding.try_into().map_err(|_| {
            BinError::new(
                Some(pos),
                BinErrorKind::Seek {
                    kind: SeekKind::Pad,
                    value: self.padding,
                },
            )
        })?;

        let value = self.f.read(reader, endian, args)?;
        reader.bin_seek(SeekFrom::Current(padding), pos + self.padding as u64)?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses() {
        let mut data = Cursor::new(vec![0x00, 0x52, 0x4F, 0xEE, 0x85, 0x00, 0x00, 0x00]);
        // Some padding to check the position of too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        let result = u32::reader().pad_after(3).read(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(0x524FEE85));
    }

    #[test]
    fn pads_for_the_next_value() {
        let mut data = Cursor::new(vec![0x00, 0x52, 0x4F, 0xEE, 0x85, 0x00, 0x00, 0x00]);
        // Some padding to check the position of too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        let _ = u32::reader().pad_after(3).read(&mut data, Endian::Big, ());
        assert_eq!(data.bin_stream_position(), Ok(8));
    }

    #[test]
    fn parses_padding_out_of_bounds() {
        let mut data = Cursor::new(vec![0x06]);

        let result = u8::reader().pad_after(300).read(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn fails_if_pad_is_too_large() {
        let mut data = Cursor::new(vec![0x00, 0x06]);
        // Some padding to check the position of too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        let result = u8::reader()
            // We can only pad by `i64::MAX`
            .pad_after(i64::MAX as usize + 1)
            .read(&mut data, Endian::Big, ());
        assert_eq!(
            result,
            Err(BinError::new(
                Some(1),
                BinErrorKind::Seek {
                    kind: SeekKind::Pad,
                    value: 9223372036854775808,
                },
            ))
        );
    }
}
