use std::borrow::Cow;
use std::error;
use std::fmt;
use std::io;

/// Represents an error encountered while parsing an MO file.
#[derive(Debug)]
pub enum Error {
    /// An incorrect magic number has been encountered
    BadMagic,
    /// An invalid byte sequence for the given encoding has been encountered
    DecodingError,
    /// An unexpected EOF occured
    Eof,
    /// An I/O error occured
    Io(io::Error),
    /// Incorrect syntax encountered while parsing the meta information
    MalformedMetadata,
    /// Meta information string was not the first string in the catalog
    MisplacedMetadata,
    /// Invalid Plural-Forms metadata
    PluralParsing,
    /// An unknown encoding was specified in the metadata
    UnknownEncoding,
}
use self::Error::*;

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BadMagic => write!(fmt, "bad magic number"),
            DecodingError => write!(fmt, "invalid byte sequence in a string"),
            Eof => write!(fmt, "unxpected end of file"),
            Io(ref err) => err.fmt(fmt),
            MalformedMetadata => write!(fmt, "metadata syntax error"),
            MisplacedMetadata => write!(fmt, "misplaced metadata"),
            UnknownEncoding => write!(fmt, "unknown encoding specified"),
            PluralParsing => write!(fmt, "invalid plural expression"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Error {
        Io(inner)
    }
}

impl From<Cow<'static, str>> for Error {
    fn from(_: Cow<'static, str>) -> Error {
        DecodingError
    }
}
