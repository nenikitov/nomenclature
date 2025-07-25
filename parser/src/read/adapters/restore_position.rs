use std::io::{Read, Seek, SeekFrom};

use crate::prelude::*;

/// Read the value without advancing the stream.
pub struct ReadRestorePosition<'f, F> {
    f: &'f mut F,
}

impl<'f, F> ReadRestorePosition<'f, F> {
    pub(super) fn new(f: &'f mut F) -> Self {
        Self { f }
    }
}

impl<F, Reader, Args, Out> BinReadCollect<Reader, Args, Out> for ReadRestorePosition<'_, F>
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

        let value = self.f.collect(reader, endian, args)?;
        reader.bin_seek(SeekFrom::Start(pos), pos)?;

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
        let mut data = Cursor::new(vec![0x00, 0x73]);
        // Some padding to check the position of too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());
        let result = u8::reader()
            .restore_position()
            .collect(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(0x73));
    }

    #[test]
    fn restores_position_after_parsing() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x3E]);
        // Some padding to check the position of too
        let _ = u16::reader().collect(&mut data, Endian::Big, ());

        let result = u8::reader()
            .restore_position()
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(_));
        assert_eq!(data.bin_stream_position(), Ok(2));
    }
}
