use std::io::{Read, Seek, SeekFrom};

use crate::{prelude::*, utils::error::SeekKind};

/// Skip an amount of bytes before a value.
pub struct PadBefore<F> {
    f: F,
    padding: usize,
}

impl<F> PadBefore<F> {
    pub(super) fn new(f: F, padding: usize) -> Self {
        Self { f, padding }
    }
}

impl<F, Reader, Args, Out> BinRead<Reader, Args, Out> for PadBefore<F>
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

        reader.bin_seek(SeekFrom::Current(padding), pos + self.padding as u64)?;
        self.f.read(reader, endian, args)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, ErrorKind};

    use super::*;

    #[test]
    fn parses() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x76]);
        // Some padding to check the position of too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        // Some padding to check the position of too
        let result = u8::reader().pad_before(5).read(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(0x76));
    }

    #[test]
    fn fails_pad_out_of_bounds() {
        let mut data = Cursor::new(vec![0x00, 0x1E]);
        // Some padding to check the position of too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        let result = u8::reader().pad_before(10).read(&mut data, Endian::Big, ());
        assert_eq!(
            result,
            Err(BinError::new(
                Some(11),
                BinErrorKind::Io(ErrorKind::UnexpectedEof),
            ))
        );
    }

    #[test]
    fn fails_if_pad_is_too_large() {
        let mut data = Cursor::new(vec![0x00, 0x1E]);
        // Some padding to check the position of too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        let result = u8::reader()
            // We can only pad by `i64::MAX`
            .pad_before(i64::MAX as usize + 1)
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
