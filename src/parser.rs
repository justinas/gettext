extern crate byteorder;

use std::error;
use std::fmt;
use std::io;
use std::mem;

use self::byteorder::{ByteOrder, BigEndian, LittleEndian};

use super::Catalog;

/// Represents an error encountered while parsing an MO file.
#[derive(Debug)]
pub enum Error {
    /// An incorrect magic number has been encountered
    BadMagic,
    /// An unexpected EOF occured
    Eof,
    /// An I/O error occured
    Io(io::Error),
}
use Error::*;

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            BadMagic => "bad magic number",
            Eof => "unxpected end of file",
            Io(ref err) => err.description(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let self_err: &error::Error = self;
        write!(fmt, "{}", self_err.description())
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Error {
        Io(inner)
    }
}

/// According to the given magic number of a MO file,
/// returns the function which reads a `u32` in the relevant endianness.
fn get_read_u32_fn(magic: &[u8]) -> Option<fn(&[u8]) -> u32> {
    if magic == [0xde, 0x12, 0x04, 0x95] {
        Some(LittleEndian::read_u32)
    } else if magic == [0x95, 0x04, 0x12, 0xde] {
        Some(BigEndian::read_u32)
    } else {
        None
    }
}

fn parse_catalog<R: io::Read>(mut file: R) -> Result<Catalog, Error> {
    let mut contents = vec![];
    let n = try!(file.read_to_end(&mut contents));
    if n < 28 {
        return Err(Eof);
    }
    let read_u32 = match get_read_u32_fn(&contents[0..4]) {
        Some(f) => f,
        None => return Err(BadMagic),
    };
    Ok(Catalog::new())
}

#[test]
fn test_get_read_u32_fn() {
    assert!(get_read_u32_fn(&[]).is_none());
    assert!(get_read_u32_fn(&[0xde, 0x12, 0x04, 0x95, 0x00]).is_none());

    {
        let le_ptr: *const ();
        let ret_ptr;
        unsafe {
            le_ptr = mem::transmute(LittleEndian::read_u32);
            ret_ptr = mem::transmute(get_read_u32_fn(&[0xde, 0x12, 0x04, 0x95]).unwrap());
        }
        assert_eq!(le_ptr, ret_ptr);
    }

    {
        let be_ptr: *const ();
        let ret_ptr;
        unsafe {
            be_ptr = mem::transmute(BigEndian::read_u32);
            ret_ptr = mem::transmute(get_read_u32_fn(&[0x95, 0x04, 0x12, 0xde]).unwrap());
        }
        assert_eq!(be_ptr, ret_ptr);
    }
}

#[test]
fn test_parse_catalog() {
    macro_rules! assert_variant {
        ($value:expr, $variant:path) => {
            match $value {
                $variant => (),
                _ => panic!("Expected {}, got {}", stringify!($variant), $value),
            }
        };
    }

    let fluff: &[u8] = &[0; 24]; // zeros to pad our magic test cases to satisfy the length requirements

    {
        let mut reader = vec![1u8, 2, 3];
        reader.extend(fluff);
        let err = parse_catalog(&reader[..]).unwrap_err();
        assert_variant!(err, Eof);
    }

    {
        let mut reader = vec![1u8, 2, 3, 4];
        reader.extend(fluff);
        let err = parse_catalog(&reader[..]).unwrap_err();
        assert_variant!(err, BadMagic);
    }

    {
        let mut reader = vec![0x95, 0x04, 0x12, 0xde];
        reader.extend(fluff);
        assert!(parse_catalog(&reader[..]).is_ok());
    }

    {
        let mut reader = vec![0xde, 0x12, 0x04, 0x95];
        reader.extend(fluff);
        assert!(parse_catalog(&reader[..]).is_ok());
    }
}
