use std::{
    io::{Read, Seek},
    marker::PhantomData,
};

use crate::prelude::*;

/// Map a value being read from one type to another.
pub struct ReadMap<'f, F, MapFn, _Out> {
    pub(super) f: &'f mut F,
    pub(super) map: MapFn,
    // HACK: I get unconstraint generic types in the `impl` without it
    pub(super) _out: PhantomData<_Out>,
}

impl<'f, F, MapFn, _Out> ReadMap<'f, F, MapFn, _Out> {
    pub(super) fn new(f: &'f mut F, map: MapFn) -> Self {
        Self {
            f,
            map,
            _out: PhantomData,
        }
    }
}

impl<F, MapFn, Reader, Args, Out, Out2> BinReadCollect<Reader, Args, Out2>
    for ReadMap<'_, F, MapFn, Out>
where
    Reader: Read + Seek,
    F: BinReadCollect<Reader, Args, Out>,
    MapFn: Fn(Out) -> Out2,
{
    fn collect_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadCollectToken,
    ) -> BinResult<Out2> {
        self.f.collect(reader, endian, args).map(&self.map)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses() {
        let mut data = Cursor::new(vec![30]);

        let result = u8::reader()
            .map(|v| v + 20)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(50));
    }

    #[test]
    fn parses_with_correct_order() {
        let mut data = Cursor::new(vec![2]);

        let result = u8::reader()
            .map(|v| v + 2)
            .map(|v| v * 2)
            .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(8));
    }
}
