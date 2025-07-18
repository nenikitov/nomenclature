use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    Big,
    Little,
}

impl Endian {
    #[cfg(target_endian = "big")]
    pub const NATIVE: Self = Endian::Big;

    #[cfg(target_endian = "little")]
    pub const NATIVE: Self = Endian::Little;
}

impl Display for Endian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endian::Big => write!(f, "Big"),
            Endian::Little => write!(f, "Little"),
        }
    }
}
