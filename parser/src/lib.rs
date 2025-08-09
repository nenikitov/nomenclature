#![warn(clippy::pedantic)]
#![allow(clippy::ignored_unit_patterns)]

pub mod helpers;
pub mod read;
pub mod utils;
pub mod write;

pub mod prelude {
    pub use super::{
        helpers::{
            marker::Marker,
            null_string::{NullStringAscii, NullStringUtf8, NullStringUtf16},
        },
        read::{BinRead, BinReadToken, BinReader, adapters::BinReadExt, impls::VecArgs},
        utils::{
            endian::Endian,
            error::{BinError, BinErrorKind, BinResult, BinResultSeek},
        },
        write::{BinWrite, BinWriteToken, BinWriter, adapters::BinWriteExt},
    };
}
