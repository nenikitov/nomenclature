use std::io::{Read, Seek};

use crate::prelude::*;

/// Repeat the parser an amount of times, each time applying different arguments, collecting the results into a vector.
///
/// Changes arguments to be an iterator outputting the arguments for an inner parser.
/// The produced vector will have the same length as this iterator, so it must be finite.
pub struct RepeatVecArgsIter<F> {
    f: F,
}

impl<F> RepeatVecArgsIter<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F, Reader, Args, Out, It> BinRead<Reader, It, Vec<Out>> for RepeatVecArgsIter<F>
where
    Reader: Read + Seek,
    F: BinRead<Reader, Args, Out>,
    It: IntoIterator<Item = Args>,
{
    fn read_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: It,
        _: BinReadToken,
    ) -> BinResult<Vec<Out>> {
        args.into_iter()
            .map(|args| self.f.read(reader, endian, args))
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
        let mut data = Cursor::new(vec![
            0x00, 0x00, 0x00, 0x00, 0xD5, 0xA0, 0xBA, 0x12, 0x2D, 0x95, 0x1B, 0x79, 0x6C, 0x5B,
        ]);
        // Some padding to check the position of too
        let _ = u32::reader().read(&mut data, Endian::Big, ());

        let addition_parser = |reader: &mut Cursor<Vec<u8>>, endian, args: u8| {
            u8::reader().read(reader, endian, ()).map(|v| v + args)
        };
        let result = addition_parser
            .repeat_vec_args_iter()
            .read(&mut data, Endian::Big, 0..10);
        assert_matches!(
            result,
            Ok(inner)
            if inner == [0xD5, 0xA1, 0xBC, 0x15, 0x31, 0x9a, 0x21, 0x80, 0x74, 0x64]
        );
    }
}
