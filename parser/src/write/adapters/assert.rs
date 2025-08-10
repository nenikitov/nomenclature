use std::{
    fmt::Debug,
    io::{Seek, Write},
};

use crate::prelude::*;

pub struct Assert<'f, F, AssertFn, MessageFn> {
    f: &'f mut F,
    assertion: AssertFn,
    message: MessageFn,
}

impl<'f, F, AssertFn, MessageFn> Assert<'f, F, AssertFn, MessageFn> {
    pub fn new(f: &'f mut F, assertion: AssertFn, message: MessageFn) -> Self {
        Self {
            f,
            assertion,
            message,
        }
    }
}

impl<'f, F, AssertFn, MessageFn, Writer, Args> BinWrite<Writer, Args>
    for Assert<'f, F, AssertFn, MessageFn>
where
    Writer: Write + Seek,
    F: BinWrite<Writer, Args>,
    AssertFn: Fn() -> bool,
    MessageFn: Fn() -> String,
{
    fn write_non_backtracking(
        &mut self,
        writer: &mut Writer,
        endian: Endian,
        args: Args,
        _: BinWriteToken,
    ) -> BinResult<()> {
        // TODO(nenikitov): How do I get inner value here?
        todo!()
    }
}
