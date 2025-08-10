use std::{
    io::{Read, Seek},
    marker::PhantomData,
};

use crate::prelude::*;

/// Map a value being read from one type to another.
pub struct Map<'f, F, MapFn, _Out> {
    f: &'f mut F,
    map: MapFn,
    // HACK: I get unconstraint generic types in the `impl` without it
    _out: PhantomData<_Out>,
}

impl<'f, F, MapFn, _Out> Map<'f, F, MapFn, _Out> {
    pub(super) fn new(f: &'f mut F, map: MapFn) -> Self {
        Self {
            f,
            map,
            _out: PhantomData,
        }
    }
}

impl<F, MapFn, Reader, Args, Out, Out2> BinRead<Reader, Args, Out2> for Map<'_, F, MapFn, Out>
where
    Reader: Read + Seek,
    F: BinRead<Reader, Args, Out>,
    MapFn: Fn(Out) -> Out2,
{
    fn read_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadToken,
    ) -> BinResult<Out2> {
        self.f.read(reader, endian, args).map(&self.map)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parses_and_maps_value() {
        let mut data = Cursor::new(vec![30]);

        let result = u8::reader()
            .map(|v| v + 20)
            .read(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(50));
    }

    #[test]
    fn parses_and_maps_value_with_correct_order() {
        let mut data = Cursor::new(vec![2]);

        let result = u8::reader()
            .map(|v| v + 2)
            .map(|v| v * 2)
            .read(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(8));
    }
}
