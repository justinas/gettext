use std::default::Default;
use std::io;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use encoding::label::encoding_from_whatwg_label;
use encoding::types::DecoderTrap::Strict;
use encoding::types::EncodingRef;

use crate::metadata::parse_metadata;
use crate::plurals::{Ast, Resolver};
use crate::Error::{self, *};
use crate::{Catalog, Message};

#[allow(non_upper_case_globals)]
static utf8_encoding: EncodingRef = &encoding::codec::utf_8::UTF8Encoding;

/// ParseOptions allows setting options for parsing MO catalogs.
///
/// # Examples
/// ```ignore
/// use std::fs::File;
/// use encoding::all::ISO_8859_1;
///
/// let file = File::open("french.mo").unwrap();
/// let catalog = ParseOptions::new().force_encoding(ISO_8859_1).parse(file).unwrap();
/// ```
#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct ParseOptions {
    force_encoding: Option<EncodingRef>,
    force_plural: Option<fn(i64) -> usize>,
}

impl ParseOptions {
    /// Returns a new instance of ParseOptions with default options.
    pub fn new() -> Self {
        Default::default()
    }

    /// Tries to parse the catalog from the given reader using the specified options.
    pub fn parse<R: io::Read>(self, reader: R) -> Result<Catalog, Error> {
        parse_catalog(reader, self)
    }

    /// Forces a use of a specific encoding
    /// when parsing strings from a catalog.
    /// If this option is not enabled,
    /// the parser tries to use the encoding specified in the metadata
    /// or UTF-8 if metadata is non-existent.
    pub fn force_encoding(mut self, encoding: EncodingRef) -> Self {
        self.force_encoding = Some(encoding);
        self
    }

    /// Forces a use of the given plural formula
    /// for deciding the proper plural form for a message.
    /// If this option is not enabled,
    /// the parser tries to use the plural formula specified in the metadata
    /// or `n != 1` if metadata is non-existent.
    pub fn force_plural(mut self, plural: fn(i64) -> usize) -> Self {
        self.force_plural = Some(plural);
        self
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

pub fn parse_catalog<R: io::Read>(mut file: R, opts: ParseOptions) -> Result<Catalog, Error> {
    let mut contents = vec![];
    let n = file.read_to_end(&mut contents)?;
    if n < 28 {
        return Err(Eof);
    }

    let read_u32 = get_read_u32_fn(&contents[0..4]).ok_or(BadMagic)?;

    // ignore hashing tables (bytes at 20..28)
    let num_strings = read_u32(&contents[8..12]) as usize;
    let mut off_otable = read_u32(&contents[12..16]) as usize;
    let mut off_ttable = read_u32(&contents[16..20]) as usize;
    if n < off_otable || n < off_ttable {
        return Err(Eof);
    }

    let mut catalog = Catalog::new();
    if let Some(f) = opts.force_plural {
        catalog.resolver = Resolver::Function(f);
    }
    let mut encoding = opts.force_encoding.unwrap_or(utf8_encoding);

    for i in 0..num_strings {
        // Parse the original string
        if n < off_otable + 8 {
            return Err(Eof);
        }
        let len = read_u32(&contents[off_otable..off_otable + 4]) as usize;
        let off = read_u32(&contents[off_otable + 4..off_otable + 8]) as usize;
        // +1 compensates for the ending NUL byte which is not included in length
        if n < off + len + 1 {
            return Err(Eof);
        }
        let mut original = &contents[off..=off + len];
        // check for context
        let context = match original.iter().position(|x| *x == 4) {
            Some(idx) => {
                let ctx = &original[..idx];
                original = &original[idx + 1..];
                Some(encoding.decode(ctx, Strict)?)
            }
            None => None,
        };
        // extract msg_id singular and plural
        let (id, plural) = match original
            .iter()
            .position(|x| *x == 0)
            .map(|i| (&original[..i], &original[i + 1..]))
        {
            Some((b_singular, b_plural)) => {
                if b_plural.is_empty() {
                    (encoding.decode(b_singular, Strict)?, None)
                } else {
                    let plural_string = encoding.decode(b_plural, Strict)?;
                    let trimmed_plural = plural_string.trim_end_matches('\0');
                    (
                        encoding.decode(b_singular, Strict)?,
                        Some(trimmed_plural.to_string()),
                    )
                }
            }
            None => return Err(Eof),
        };
        if id == "" && i != 0 {
            return Err(MisplacedMetadata);
        }

        // Parse the translation strings
        if n < off_ttable + 8 {
            return Err(Eof);
        }
        let len = read_u32(&contents[off_ttable..off_ttable + 4]) as usize;
        let off = read_u32(&contents[off_ttable + 4..off_ttable + 8]) as usize;
        // +1 compensates for the ending NUL byte which is not included in length
        if n < off + len + 1 {
            return Err(Eof);
        }
        let translated = contents[off..off + len]
            .split(|x| *x == 0)
            .map(|b| encoding.decode(b, Strict))
            .collect::<Result<Vec<_>, _>>()?;
        if id == "" {
            // Parse the metadata from the first translation string, returning early if there's an error.
            let map = parse_metadata((*translated[0]).to_string())?;
            // Set the metadata of the catalog with the parsed result.
            catalog.metadata = Some(map.clone());
            if let (Some(c), None) = (map.charset(), opts.force_encoding) {
                encoding = encoding_from_whatwg_label(c).ok_or(UnknownEncoding)?;
            }
            if opts.force_plural.is_none() {
                if let Some(p) = map.plural_forms().1 {
                    catalog.resolver = Ast::parse(p).map(Resolver::Expr)?;
                }
            }
        }

		// Checks the presence of a plural form for the message.
		// If a plural form is provided, the message is inserted into the catalog using the `with_plural` method.
		// Otherwise, the message is inserted using the default `new` method.
		if plural.is_some() {
			catalog.insert(Message::with_plural(id, context, translated, plural));
		} else {
			catalog.insert(Message::new(id, context, translated));
		}

        off_otable += 8;
        off_ttable += 8;
    }

    Ok(catalog)
}

/// The default plural resolver.
///
/// It will be used if not `Plural-Forms` header is found in the .mo file, and if
/// `ParseOptions::force_plural` was not called.
///
/// It is valid for English and similar languages: plural will be used for any quantity
/// different of 1.
pub fn default_resolver(n: i64) -> usize {
    if n == 1 {
        0
    } else {
        1
    }
}

#[test]
fn test_get_read_u32_fn() {
    assert!(get_read_u32_fn(&[]).is_none());
    assert!(get_read_u32_fn(&[0xde, 0x12, 0x04, 0x95, 0x00]).is_none());

    {
        let le_ptr = LittleEndian::read_u32 as *const ();
        let ret_ptr = get_read_u32_fn(&[0xde, 0x12, 0x04, 0x95]).unwrap() as _;
        assert_eq!(le_ptr, ret_ptr);
    }

    {
        let be_ptr = BigEndian::read_u32 as *const ();
        let ret_ptr = get_read_u32_fn(&[0x95, 0x04, 0x12, 0xde]).unwrap() as _;
        assert_eq!(be_ptr, ret_ptr);
    }
}

#[test]
fn test_parse_catalog() {
    macro_rules! assert_variant {
        ($value:expr, $variant:path) => {
            match $value {
                $variant => (),
                _ => panic!("Expected {:?}, got {:?}", $variant, $value),
            }
        };
    }

    let fluff = [0; 24]; // zeros to pad our magic test cases to satisfy the length requirements

    {
        let mut reader = vec![1u8, 2, 3];
        reader.extend(fluff.iter().cloned());
        let err = parse_catalog(&reader[..], ParseOptions::new()).unwrap_err();
        assert_variant!(err, Eof);
    }

    {
        let mut reader = vec![1u8, 2, 3, 4];
        reader.extend(fluff.iter().cloned());
        let err = parse_catalog(&reader[..], ParseOptions::new()).unwrap_err();
        assert_variant!(err, BadMagic);
    }

    {
        let mut reader = vec![0x95, 0x04, 0x12, 0xde];
        reader.extend(fluff.iter().cloned());
        assert!(parse_catalog(&reader[..], ParseOptions::new()).is_ok());
    }

    {
        let mut reader = vec![0xde, 0x12, 0x04, 0x95];
        reader.extend(fluff.iter().cloned());
        assert!(parse_catalog(&reader[..], ParseOptions::new()).is_ok());
    }

    {
        let reader: &[u8] = include_bytes!("../test_cases/1.mo");
        let catalog = parse_catalog(reader, ParseOptions::new()).unwrap();
        assert_eq!(catalog.strings.len(), 1);
        assert_eq!(
            catalog.strings["this is context\x04Text"],
            Message::new(
                "Text",
                Some("this is context"),
                vec!["Tekstas", "Tekstai"]
            )
        );
    }

    {
        let reader: &[u8] = include_bytes!("../test_cases/2.mo");
        let catalog = parse_catalog(reader, ParseOptions::new()).unwrap();
        assert_eq!(catalog.strings.len(), 2);
        assert_eq!(
            catalog.strings["Image"],
            Message::new("Image", None, vec!["Nuotrauka", "Nuotraukos"])
        );
    }

    {
        let reader: &[u8] = include_bytes!("../test_cases/invalid_utf8.mo");
        let err = parse_catalog(reader, ParseOptions::new()).unwrap_err();
        assert_variant!(err, DecodingError);
    }
}
