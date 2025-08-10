use std::io::{Read, Seek};

use array_init::try_array_init;

use crate::prelude::*;

/// Repeat the parser an amount of times, collecting the results into an array.
pub struct RepeatArray<'f, F, const N: usize> {
    f: &'f mut F,
}

impl<'f, F, const N: usize> RepeatArray<'f, F, N> {
    pub(super) fn new(f: &'f mut F) -> Self {
        Self { f }
    }
}

impl<F, const N: usize, Reader, Args, Out> BinRead<Reader, Args, [Out; N]>
    for RepeatArray<'_, F, N>
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
    ) -> BinResult<[Out; N]> {
        try_array_init(|_| self.f.read(reader, endian, args.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parses() {
        let mut data = Cursor::new(vec![0x00, 0x00, 0x9B, 0x0F, 0x74, 0xF3, 0x10, 0xC5]);
        // Some padding to check the position of too
        let _ = u16::reader().read(&mut data, Endian::Big, ());

        let result = u16::reader()
            .repeat_array::<3>()
            .read(&mut data, Endian::Big, ());
        assert_eq!(result, Ok([0x9B0F, 0x74F3, 0x10C5]));
    }
}
