pub mod adapters;
pub mod read;
pub mod utils;
pub mod write;

pub mod prelude {
    pub use super::{
        adapters::BinReadCombinator,
        read::{BinRead, BinReadCollect},
        utils::{
            endian::Endian,
            error::{BinError, BinResult},
        },
    };
}
