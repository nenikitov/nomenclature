use std::io::{Read, Seek};

use crate::prelude::*;

/// Repeat the parser an amount of times, collecting the results into an array.
pub struct ReadRepeatVecArgsIter<'f, F> {
    f: &'f mut F,
}

impl<'f, F> ReadRepeatVecArgsIter<'f, F> {
    pub fn new(f: &'f mut F) -> Self {
        Self { f }
    }
}

impl<F, Reader, Args, Out, It> BinReadCollect<Reader, It, Vec<Out>> for ReadRepeatVecArgsIter<'_, F>
where
    Reader: Read + Seek,
    F: BinReadCollect<Reader, Args, Out>,
    It: IntoIterator<Item = Args>,
{
    fn collect_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: It,
        _: BinReadCollectToken,
    ) -> BinResult<Vec<Out>> {
        args.into_iter()
            .map(|args| self.f.collect(reader, endian, args))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses_values() {
        let mut data = Cursor::new(vec![
            0x00, 0x00, 0x00, 0x00, 0xD5, 0xA0, 0xBA, 0x12, 0x2D, 0x95, 0x1B, 0x79, 0x6C, 0x5B,
        ]);
        // Some padding to check the position of too
        let _ = u32::reader().collect(&mut data, Endian::Big, ());
        let mut addition_parser = |reader: &mut Cursor<Vec<u8>>, endian, args: u8| {
            u8::reader().collect(reader, endian, ()).map(|v| v + args)
        };
        let result = addition_parser
            .repeat_vec_args_iter()
            .collect(&mut data, Endian::Big, 0..10);
        assert_matches!(
            result,
            Ok(inner)
            if inner == [0xD5, 0xA1, 0xBC, 0x15, 0x31, 0x9a, 0x21, 0x80, 0x74, 0x64]
        );
    }
}
