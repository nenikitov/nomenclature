use std::io::{Cursor, Read, Seek};

use nomenclature::prelude::*;

const DATA: &[u8; 0xB0] = include_bytes!("vfs.bin");

#[derive(Debug, PartialEq)]
struct VfsFile {
    message: String,
    files: Vec<Vec<u8>>,
}

impl BinReader for VfsFile {
    type Args<'a> = ();
    type Out = Self;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, _| {
            let _ = <[u8; 4]>::reader()
                .assert(|v| v == b"VFS\0", |v| format!("Bad magic {v:?}"))
                .read(reader, endian, ())?;

            let message = NullStringAscii::reader()
                .pad_after_to(24)
                .read(reader, endian, ())?;

            let entries = u32::reader().read(reader, endian, ())?;

            let offsets =
                <(u32, u32)>::reader()
                    .repeat_vec(entries as usize)
                    .read(reader, endian, ())?;

            let files = (|reader: &mut Reader, endian, &(offset, len)| {
                u8::reader()
                    .repeat_vec(len as usize)
                    .seek_before(offset as usize)
                    .read(reader, endian, ())
            })
            .repeat_vec_args_iter()
            .read(reader, endian, &offsets)?;

            Ok(Self { message, files })
        }
    }
}

#[test]
fn main() {
    let mut data = Cursor::new(DATA);
    let result = VfsFile::reader().read(&mut data, Endian::Little, ());

    assert_eq!(
        result,
        Ok(VfsFile {
            message: "Some file description".to_string(),
            files: vec![
                b"First".to_vec(),
                b"Second".to_vec(),
                b"Third".to_vec(),
                b"Fourth".to_vec(),
                b"Fifth".to_vec(),
                b"Sixth".to_vec(),
                b"Seventh".to_vec(),
                b"Eighth".to_vec(),
                b"Ninth".to_vec(),
                b"Tenth".to_vec(),
            ]
        })
    )
}
