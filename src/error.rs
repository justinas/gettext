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
use Error::*;

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            BadMagic => "bad magic number",
            DecodingError => "invalid byte sequence in a string",
            Eof => "unxpected end of file",
            Io(ref err) => err.description(),
            MalformedMetadata => "metadata syntax error",
            MisplacedMetadata => "misplaced metadata",
            UnknownEncoding => "unknown encoding specified",
            PluralParsing => "invalid plural expression",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let self_err: &dyn error::Error = self;
        write!(fmt, "{}", self_err.description())
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
