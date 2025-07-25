use std::io::{Read, Seek};

use crate::prelude::*;

/// Fail if a specified condition on a parsed value doesn't pass.
pub struct ReadAssert<'f, F, AssertFn, MessageFn> {
    f: &'f mut F,
    assertion: AssertFn,
    message: MessageFn,
}

impl<'f, F, AssertFn, MessageFn> ReadAssert<'f, F, AssertFn, MessageFn> {
    pub(super) fn new(f: &'f mut F, assertion: AssertFn, message: MessageFn) -> Self {
        Self {
            f,
            assertion,
            message,
        }
    }
}

impl<F, AssertFn, MessageFn, Reader, Args, Out> BinReadCollect<Reader, Args, Out>
    for ReadAssert<'_, F, AssertFn, MessageFn>
where
    Reader: Read + Seek,
    F: BinReadCollect<Reader, Args, Out>,
    AssertFn: Fn(&Out) -> bool,
    MessageFn: Fn(&Out) -> String,
{
    fn collect_non_backtracking(
        &mut self,
        reader: &mut Reader,
        endian: Endian,
        args: Args,
        _: BinReadCollectToken,
    ) -> BinResult<Out> {
        let pos = reader.stream_position()?;

        let value = (self.f).collect(reader, endian, args)?;
        if !(self.assertion)(&value) {
            return Err(BinErrorKind::AssertionFailed {
                pos,
                message: (self.message)(&value),
            });
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use assert_matches::*;

    use super::*;

    #[test]
    fn parses_value_if_assertion_returns_true() {
        let mut data = Cursor::new(vec![0x3F, 0xDA]);

        let result =
            u16::reader()
                .assert(|_| true, |_| unreachable!())
                .collect(&mut data, Endian::Big, ());
        assert_matches!(result, Ok(0x3FDA));
    }

    #[test]
    fn fails_if_assertion_returns_false() {
        let mut data = Cursor::new(vec![0x00, 0x41, 0xE1]);
        // Some padding to check the position of the error too
        let _ = u8::reader().collect(&mut data, Endian::Big, ());

        let result = u16::reader()
            .assert(|_| false, |v| format!("the value {v:X} is invalid"))
            .collect(&mut data, Endian::Big, ());
        assert_matches!(
            result,
            Err(BinErrorKind::AssertionFailed {
                pos: 1,
                message,
            }) if message == "the value 41E1 is invalid"
        );
    }
}
