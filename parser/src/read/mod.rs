mod impls;

use std::io::{Read, Seek, SeekFrom};

use crate::utils::{endian::Endian, error::BinResult};

pub trait BinRead {
    type Args<'a>;
    // TODO(nenikitov): Make this default to `Self` when `associated_type_defaults` gets stabilized
    type Out;

    fn bin_read_non_backtracking<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self::Out>;

    #[inline]
    fn bin_read<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self::Out> {
        let pos = reader.stream_position()?;

        match Self::bin_read_non_backtracking(reader, endian, args) {
            e if e.is_err() => {
                reader.seek(SeekFrom::Start(pos))?;
                e
            }
            v => v,
        }
    }
}
