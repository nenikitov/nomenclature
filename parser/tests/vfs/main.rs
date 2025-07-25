use std::io::Cursor;

use nomenclature::prelude::*;

const DATA: &[u8; 0xB0] = include_bytes!("vfs.bin");

pub struct VfsSubFile;

#[derive(Debug, PartialEq, Eq)]
pub struct VfsFile {
    message: String,
    files: Vec<Vec<u8>>,
}

impl BinReader for VfsFile {
    type Args = ();
    type Out = Self;

    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: std::io::Read + std::io::Seek,
    {
        |reader: &mut Reader, endian, _| {
            let _ = <[u8; 4]>::reader()
                .assert(|v| v == b"VFS\0", |v| format!("Bad magic {v:?}"))
                .collect(reader, endian, ())?;

            let message = NullStringAscii::reader()
                .pad_after_to(24)
                .collect(reader, endian, ())?;

            let entries = u32::reader().collect(reader, endian, ())?;

            let offsets = <(u32, u32)>::reader()
                .repeat_vec(entries as usize)
                .collect(reader, endian, ())?;

            let files = (|reader: &mut Reader, endian, &(offset, len)| {
                u8::reader()
                    .repeat_vec(len as usize)
                    .seek_before(offset as usize)
                    .collect(reader, endian, ())
            })
            .repeat_vec_args_iter()
            .collect(reader, endian, &offsets)?;

            Ok(Self { message, files })
        }
    }
}

#[test]
fn main() {
    let mut data = Cursor::new(DATA);
    let result = VfsFile::reader().collect(&mut data, Endian::Little, ());

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
