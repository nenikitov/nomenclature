use std::fmt::Display;

/// Machine byte order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    /// Bytes are ordered most to least significant.
    /// This means that a `1u16` is stored as `0x00_01`.
    Big,
    /// Bytes are ordered least to most significant.
    /// This means that a `1u16` is stored as `0x01_00`.
    Little,
}

impl Endian {
    /// Native endianness of the machine.
    #[cfg(target_endian = "big")]
    pub const NATIVE: Self = Self::Big;

    /// Native endianness of the machine.
    #[cfg(target_endian = "little")]
    pub const NATIVE: Self = Self::Little;
}

impl Display for Endian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endian::Big => write!(f, "Big"),
            Endian::Little => write!(f, "Little"),
        }
    }
}
