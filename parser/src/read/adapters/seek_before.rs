use std::io::{Read, Seek, SeekFrom};

use crate::{prelude::*, utils::error::SeekKind};

/// Position the stream before reading the value.
///
/// If succeeds, the stream is kept in the position right after the parsed value.
/// If you need to restore position, use [`BinReadExt::restore_position`].
pub struct ReadSeekBefore<'f, F> {
    f: &'f mut F,
    position: usize,
}

impl<'f, F> ReadSeekBefore<'f, F> {
    pub(super) fn new(f: &'f mut F, position: usize) -> Self {
        Self { f, position }
    }
}

impl<F, Reader, Args, Out> BinReadCollect<Reader, Args, Out> for ReadSeekBefore<'_, F>
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
        let position = self.position.try_into().map_err(|_| {
            BinError::new(
                Some(pos),
                BinErrorKind::Seek {
                    kind: SeekKind::Seek,
                    value: self.position,
                },
            )
        })?;

        reader.bin_seek(SeekFrom::Start(position), position)?;
        let value = self.f.collect(reader, endian, args)?;

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
        let mut data = Cursor::new(vec![0x00, 0xB9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // Some padding to check the position of too
        let _ = u64::reader().collect(&mut data, Endian::Big, ());
        let result = u8::reader()
            .seek_before(1)
            .collect(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(0xB9));
    }

    #[test]
    fn keeps_stream_position() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x00, 0x00, 0x5E, 0x00, 0x00, 0x00]);
        // Some padding to check the position of too
        let _ = u64::reader().collect(&mut data, Endian::Big, ());

        let result = u8::reader()
            .seek_before(4)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(_));
        assert_eq!(data.bin_stream_position(), Ok(5));
    }
}
