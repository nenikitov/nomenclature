use std::io::{Read, Seek, SeekFrom};

use crate::{prelude::*, utils::error::SeekKind};

/// Skip some bytes after the value so the stream always advances by a given amount.
///
// TODO(nenikitov): But should it fail while reading only?
/// Will not fail if the stream has ended during padding.
pub struct ReadPadAfterTo<'f, F> {
    f: &'f mut F,
    size: usize,
}

impl<'f, F> ReadPadAfterTo<'f, F> {
    pub(super) fn new(f: &'f mut F, size: usize) -> Self {
        Self { f, size }
    }
}

impl<F, Reader, Args, Out> BinReadCollect<Reader, Args, Out> for ReadPadAfterTo<'_, F>
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
        let pos = reader.bin_stream_position()?;

        // TODO(nenikitov): Test this on a 128-bit target.
        // Because `usize = u64` on 64-bit targets, a conversion between them is never an exception.
        let size_expected = self.size.try_into().map_err(|_| {
            BinError::new(
                Some(pos),
                BinErrorKind::Seek {
                    kind: SeekKind::Size,
                    value: self.size,
                },
            )
        })?;

        let value = self.f.collect(reader, endian, args)?;
        let pos_after = reader.bin_stream_position()?;

        let size = pos_after - pos;

        if size > size_expected {
            return Err(BinError::new(
                Some(pos_after),
                BinErrorKind::Size {
                    expected: self.size,
                    // TODO(nenikitov): There is surely a clean way of doing this
                    #[allow(clippy::cast_possible_truncation)]
                    got: size as usize,
                },
            ));
        }

        reader.bin_seek(SeekFrom::Start(pos + size_expected), pos + size_expected)?;

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
        let mut data = Cursor::new(vec![0x00, 0xEB, 0x73, 0x7B, 0x2A, 0x00, 0x00]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());

        let result = u32::reader()
            .pad_after_to(6)
            .collect(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(0xEB737B2A));
    }

    #[test]
    fn pads_for_the_next_value() {
        let mut data = Cursor::new(vec![0x00, 0x06, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());

        let _ = u16::reader()
            .pad_after_to(7)
            .collect(&mut data, Endian::Big, ());
        assert_eq!(data.bin_stream_position(), Ok(8));
    }

    #[test]
    fn parses_padding_out_of_bounds() {
        let mut data = Cursor::new(vec![0x29]);

        let result = u8::reader()
            .pad_after_to(300)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn fails_if_value_is_too_large() {
        let mut data = Cursor::new(vec![0x00, 0x3E, 0xA2]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());

        let result = u16::reader()
            // We can only pad to `u64::MAX`
            .pad_after_to(1)
            .collect(&mut data, Endian::Big, ());
        assert_eq!(
            result,
            Err(BinError::new(
                Some(3),
                BinErrorKind::Size {
                    expected: 1,
                    got: 2,
                },
            ))
        );
    }
}
