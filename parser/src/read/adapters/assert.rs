use std::{
    fmt::Debug,
    io::{Read, Seek},
};

use crate::prelude::*;

/// Fail if a specified condition on a parsed value doesn't pass.
pub struct Assert<F, AssertFn, MessageFn> {
    f: F,
    assertion: AssertFn,
    message: MessageFn,
}

impl<'f, F, AssertFn, MessageFn> Assert<F, AssertFn, MessageFn> {
    pub(super) fn new(f: F, assertion: AssertFn, message: MessageFn) -> Self {
        Self {
            f,
            assertion,
            message,
        }
    }
}

impl<F, AssertFn, MessageFn, Reader, Args, Out> BinRead<Reader, Args, Out>
    for Assert<F, AssertFn, MessageFn>
where
    Reader: Read + Seek,
    F: BinRead<Reader, Args, Out>,
    AssertFn: Fn(&Out) -> bool,
    MessageFn: Fn(&Out) -> String,
    Out: Debug,
{
    fn read_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadToken,
    ) -> BinResult<Out> {
        let pos = reader.bin_stream_position()?;

        let value = self.f.read(reader, endian, args)?;
        if !(self.assertion)(&value) {
            return Err(BinError::new(
                Some(pos),
                BinErrorKind::Assertion {
                    value: format!("{value:?}"),
                    message: (self.message)(&value),
                },
            ));
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parses_value_if_assertion_returns_true() {
        let mut data = Cursor::new(vec![0x3F, 0xDA]);

        let result =
            u16::reader()
                .assert(|_| true, |_| unreachable!())
                .read(&mut data, Endian::Big, ());
        assert_eq!(result, Ok(0x3FDA));
    }

    #[test]
    fn fails_if_assertion_returns_false() {
        let mut data = Cursor::new(vec![0x00, 0x41, 0xE1]);
        // Some padding to check the position of the error too
        let _ = u8::reader().read(&mut data, Endian::Big, ());

        let result = u16::reader()
            .assert(|_| false, |v| format!("the value {v:X} is invalid"))
            .read(&mut data, Endian::Big, ());
        assert_eq!(
            result,
            Err(BinError::new(
                Some(1,),
                BinErrorKind::Assertion {
                    value: "16865".to_string(),
                    message: "the value 41E1 is invalid".to_string(),
                },
            ))
        )
    }
}
