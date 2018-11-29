use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    InvalidPath,
    KeyAlreadyPresent,
    ParsingError(String),
    IoError(io::Error),
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidPath => write!(f, "Path is Invalid"),
            Error::KeyAlreadyPresent => write!(f, "Key is already present in the index"),
            Error::ParsingError(s) => write!(f, "Unable to parse {}", s),
            Error::IoError(e) => e.fmt(f),
        }
    }
}
