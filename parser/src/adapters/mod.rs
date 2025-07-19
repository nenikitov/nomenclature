pub mod assert;

use std::io::{Read, Seek};

use crate::read::BinReadCollect;

pub trait BinReadCombinator<Reader, Args, Out>: BinReadCollect<Reader, Args, Out>
where
    Reader: Read + Seek,
{
}

impl<T, Reader, Args, Out> BinReadCombinator<Reader, Args, Out> for T
where
    T: BinReadCollect<Reader, Args, Out>,
    Reader: Read + Seek,
{
}
