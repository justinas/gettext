extern crate encoding;

use std::fmt;

use self::encoding::types::EncodingRef;

/// ParseOptions allows setting options for parsing MO catalogs.
#[allow(missing_debug_implementations)]
pub struct ParseOptions {
    force_encoding: Option<EncodingRef>,
}

impl ParseOptions {
    /// Returns a new instance of ParseOptions with default options.
    pub fn new() -> Self {
        ParseOptions { force_encoding: None }
    }

    /// Forces a use of a specific encoding
    /// when parsing strings from a catalog.
    /// If this option is not enabled,
    /// the parser tries to use the encoding specified in the metadata
    /// or UTF-8 if metadata is non-existent.
    pub fn force_encoding(&mut self, encoding: EncodingRef) -> &mut Self {
        self.force_encoding = Some(encoding);
        self
    }
}
