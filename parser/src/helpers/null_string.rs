use std::{
    io::{Read, Seek},
    iter,
};

use crate::prelude::*;

/// A dummy struct that parses null-terminated ASCII strings and converts them to [`String`].
pub struct NullStringAscii;

impl BinReader for NullStringAscii {
    type Args = ();
    type Out = String;

    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, args| {
            iter::from_fn(|| match u8::reader().collect(reader, endian, args) {
                Ok(0) => None,
                Ok(v) => Some(Ok(v as char)),
                Err(e) => Some(Err(e)),
            })
            .collect()
        }
    }
}

/// A dummy struct that parses null-terminated UTF-8 strings and converts them to [`String`].
pub struct NullStringUtf8;

impl BinReader for NullStringUtf8 {
    type Args = ();
    type Out = String;

    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, args| {
            let bytes: Vec<_> =
                iter::from_fn(|| match u8::reader().collect(reader, endian, args) {
                    Ok(0) => None,
                    Ok(v) => Some(Ok(v)),
                    Err(e) => Some(Err(e)),
                })
                .collect::<BinResult<_>>()?;

            Ok(String::from_utf8(bytes)?)
        }
    }
}

pub struct NullStringUtf16;

/// A dummy struct that parses null-terminated UTF-16 strings and converts them to [`String`].
impl BinReader for NullStringUtf16 {
    type Args = ();
    type Out = String;

    fn reader<Reader>() -> impl BinReadCollect<Reader, Self::Args, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, args| {
            let bytes: Vec<_> =
                iter::from_fn(|| match u16::reader().collect(reader, endian, args) {
                    Ok(0) => None,
                    Ok(v) => Some(Ok(v)),
                    Err(e) => Some(Err(e)),
                })
                .collect::<BinResult<_>>()?;

            Ok(String::from_utf16(&bytes)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    mod ascii {
        use super::*;

        #[test]
        fn parses() {
            let mut data = Cursor::new(vec![
                0x00, 0x00, 0x32, 0x30, 0x20, 0xF7, 0x20, 0x35, 0x20, 0x3D, 0x20, 0x34, 0x00,
            ]);
            // Some padding to check the position of too
            let _ = u16::reader().collect(&mut data, Endian::Big, ());

            let result = NullStringAscii::reader().collect(&mut data, Endian::Big, ());
            assert_matches!(
                result,
                Ok(inner)
                if inner == "20 ÷ 5 = 4"
            );
        }

        #[test]
        fn stops_at_first_null_terminator() {
            let mut data = Cursor::new(vec![0x00, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0x01]);
            // Some padding to check the position of too
            let _ = u8::reader().collect(&mut data, Endian::Big, ());

            let result = NullStringAscii::reader().collect(&mut data, Endian::Big, ());
            assert_matches!(result, Ok(_));
            assert_matches!(data.stream_position(), Ok(7));
        }
    }

    mod utf_8 {
        use super::*;

        #[test]
        fn parses() {
            let mut data = Cursor::new(vec![
                0x00, 0x00, 0x00, 0x00, 0x32, 0x30, 0x20, 0xC3, 0xB7, 0x20, 0x35, 0x20, 0x3D, 0x20,
                0x34, 0x00,
            ]);
            // Some padding to check the position of too
            let _ = u32::reader().collect(&mut data, Endian::Big, ());

            let result = NullStringUtf8::reader().collect(&mut data, Endian::Big, ());
            assert_matches!(
                result,
                Ok(inner)
                if inner == "20 ÷ 5 = 4"
            );
        }

        #[test]
        fn stops_at_first_null_terminator() {
            let mut data = Cursor::new(vec![0x00, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x00, 0x01]);
            // Some padding to check the position of too
            let _ = u8::reader().collect(&mut data, Endian::Big, ());

            let result = NullStringUtf8::reader().collect(&mut data, Endian::Big, ());
            assert_matches!(result, Ok(_));
            assert_matches!(data.stream_position(), Ok(8));
        }
    }

    mod utf_16 {
        use super::*;

        #[test]
        fn parses() {
            let mut data = Cursor::new(vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x32, 0x00, 0x30, 0x00, 0x20, 0x00, 0xF7, 0x00, 0x20,
                0x00, 0x35, 0x00, 0x20, 0x00, 0x3D, 0x00, 0x20, 0x00, 0x34, 0x00, 0x00,
            ]);
            // Some padding to check the position of too
            let _ = u32::reader().collect(&mut data, Endian::Big, ());

            let result = NullStringUtf16::reader().collect(&mut data, Endian::Big, ());
            assert_matches!(
                result,
                Ok(inner)
                if inner == "20 ÷ 5 = 4"
            );
        }

        #[test]
        fn stops_at_first_null_terminator() {
            let mut data = Cursor::new(vec![
                0x00, 0x00, 0x48, 0x00, 0x65, 0x00, 0x79, 0x00, 0x00, 0x01,
            ]);
            // Some padding to check the position of too
            let _ = u8::reader().collect(&mut data, Endian::Big, ());

            let result = NullStringUtf16::reader().collect(&mut data, Endian::Big, ());
            assert_matches!(result, Ok(_));
            assert_matches!(data.stream_position(), Ok(9));
        }
    }
}
