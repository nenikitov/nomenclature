pub mod adapters;
pub mod read;
pub mod utils;
pub mod write;

pub mod prelude {
    pub use super::{
        adapters::BinReadAdapter,
        read::{BinRead, BinReadCollect, BinReadToken},
        utils::{
            endian::Endian,
            error::{BinError, BinResult},
        },
    };
}
