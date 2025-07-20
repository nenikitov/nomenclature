#![warn(clippy::pedantic)]

pub mod adapters;
pub mod read;
pub mod utils;
pub mod write;

pub mod prelude {
    pub use super::{
        adapters::BinReadExt,
        read::{BinReadCollect, BinReadCollectToken, BinReader},
        utils::{
            endian::Endian,
            error::{BinErrorKind, BinResult},
        },
    };
}
