use std::io::Error;

#[derive(Debug)]
pub enum BinError {
    AssertionFailed { pos: u64, message: String },
    Io(Error),
}

impl From<Error> for BinError {
    fn from(value: Error) -> Self {
        Self::Io(value)
    }
}

pub type BinResult<T> = Result<T, BinError>;
