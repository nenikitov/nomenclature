use crate::read::BinRead;

pub mod assert;

pub trait BinReadExt<T>
where
    T: BinRead,
{
}

impl<T> BinReadExt<T> for T where T: BinRead {}
