use std::io::{Read, Seek};

use crate::prelude::*;

/// Repeat the parser an amount of times, reading the results into a vector.
pub struct ReadRepeatVec<'f, F> {
    f: &'f mut F,
    len: usize,
}

impl<'f, F> ReadRepeatVec<'f, F> {
    pub fn new(f: &'f mut F, len: usize) -> Self {
        Self { f, len }
    }
}

impl<F, Reader, Args, Out> BinRead<Reader, Args, Vec<Out>> for ReadRepeatVec<'_, F>
where
    Reader: Read + Seek,
    F: BinRead<Reader, Args, Out>,
    Args: Clone,
{
    fn read_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadToken,
    ) -> BinResult<Vec<Out>> {
        (0..self.len)
            .map(|_| self.f.read(reader, endian, args.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x9F, 0xAF, 0x70, 0x63, 0x86, 0x81, 0xE4]);
        // Some padding to check the position of too
        let _ = u16::reader().read(&mut data, Endian::Big, ());

        let result = u8::reader().repeat_vec(7).read(&mut data, Endian::Big, ());
        assert_matches!(
            result,
            Ok(inner)
            if inner == [0x9F, 0xAF, 0x70, 0x63, 0x86, 0x81, 0xE4]
        );
    }
}
