use std::io::{Read, Seek};

use array_init::try_array_init;

use crate::prelude::*;

/// Repeat the parser an amount of times, collecting the results into an array.
pub struct ReadRepeatArray<'f, F, const N: usize> {
    f: &'f mut F,
}

impl<'f, F, const N: usize> ReadRepeatArray<'f, F, N> {
    pub(super) fn new(f: &'f mut F) -> Self {
        Self { f }
    }
}

impl<F, const N: usize, Reader, Args, Out> BinReadCollect<Reader, Args, [Out; N]>
    for ReadRepeatArray<'_, F, N>
where
    Reader: Read + Seek,
    F: BinReadCollect<Reader, Args, Out>,
    Args: Clone,
{
    fn collect_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadCollectToken,
    ) -> BinResult<[Out; N]> {
        try_array_init(|_| self.f.collect(reader, endian, args.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses_values() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x9B, 0x0F, 0x74, 0xF3, 0x10, 0xC5]);
        // Some padding to check the position of too
        let _ = u16::reader().collect(&mut data, Endian::Big, ());
        let result = u16::reader()
            .repeat_array::<3>()
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok([0x9B0F, 0x74F3, 0x10C5]))
    }
}
