use std::io::{Read, Seek, SeekFrom};

use crate::{prelude::*, utils::error::SeekKind};

/// Skip an amount of bytes after a value.
///
// TODO(nenikitov): But should it fail while reading only?
/// Will not fail if the stream has ended during padding.
pub struct ReadPadBefore<'f, F> {
    pub(super) f: &'f mut F,
    pub(super) padding: usize,
}

impl<'f, F> ReadPadBefore<'f, F> {
    pub fn new(f: &'f mut F, padding: usize) -> Self {
        Self { f, padding }
    }
}

impl<F, Reader, Args, Out> BinReadCollect<Reader, Args, Out> for ReadPadBefore<'_, F>
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

        reader.seek(SeekFrom::Current(padding))?;
        (self.f).collect(reader, endian, args)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses_value() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x76]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());
        // Some padding to check the position of too
        let result = u8::reader()
            .pad_before(5)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(0x76))
    }

    #[test]
    fn fails_pad_out_of_bounds() {
        let mut data = Cursor::new(vec![0x00, 0x1E]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());
        let result = u8::reader()
            .pad_before(10)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(
            result,
            Err(BinErrorKind::Io(err))
            if err.kind() == std::io::ErrorKind::UnexpectedEof
        )
    }

    #[test]
    fn fails_if_pad_is_too_large() {
        let mut data = Cursor::new(vec![0x00, 0x1E]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());
        let result = u8::reader()
            // We can only pad by `i64::MAX`
            .pad_before(i64::MAX as usize + 1)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(
            result,
            Err(BinErrorKind::InvalidSeek {
                pos: 1,
                kind: SeekKind::Pad,
                value: 9223372036854775808
            })
        )
    }
}
