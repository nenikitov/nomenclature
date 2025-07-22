#![warn(clippy::pedantic)]

pub mod helpers;
pub mod read;
pub mod utils;
pub mod write;

pub mod prelude {
    pub use super::{
        helpers::marker::Marker,
        read::{BinReadCollect, BinReadCollectToken, BinReader, adapters::BinReadExt},
        utils::{
            endian::Endian,
            error::{BinErrorKind, BinResult},
        },
    };
}
