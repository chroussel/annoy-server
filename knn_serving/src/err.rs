use std::fmt;

#[derive(Debug)]
pub enum Error {
    NoIndexLoaded(String),
    SerializationError(capnp::Error),
    DimensionError(usize, usize),
    CancelledFuture,
    IoError(::std::io::Error),
    ParsingError(String),
    NoProductVectorFound(i64),
    IndexError(annoy_rs::err::Error),
    HyperError(hyper::Error),
    HttpError(hyper::http::Error),
    NotFound,
    JsonParsingError(serde_json::Error),
}

impl From<::capnp::Error> for Error {
    fn from(err: ::capnp::Error) -> Self {
        Error::SerializationError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<annoy_rs::err::Error> for Error {
    fn from(err: annoy_rs::err::Error) -> Self {
        Error::IndexError(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::HyperError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonParsingError(err)
    }
}

impl From<hyper::http::Error> for Error {
    fn from(err: hyper::http::Error) -> Self {
        Error::HttpError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoIndexLoaded(value) => write!(f, "No index loaded for {}", value),
            Error::DimensionError(expected, result) => write!(
                f,
                "Dimension does not match expected {} got {}",
                expected, result
            ),
            Error::CancelledFuture => write!(f, "Operation has been cancelled"),
            Error::IoError(io) => io.fmt(f),
            Error::ParsingError(value) => write!(f, "Error parsing {}", value),
            Error::SerializationError(value) => value.fmt(f),
            Error::HttpError(value) => value.fmt(f),
            Error::NotFound => write!(f, "Not found"),
            Error::IndexError(err) => err.fmt(f),
            Error::HyperError(err) => err.fmt(f),
            Error::JsonParsingError(err) => err.fmt(f),
            Error::NoProductVectorFound(value) => value.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
