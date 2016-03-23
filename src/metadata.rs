use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use super::Error;
use Error::MalformedMetadata;

#[derive(Debug)]
pub struct MetadataMap<'a>(HashMap<&'a str, &'a str>);

impl<'a> MetadataMap<'a> {
    /// Returns a string that indicates the character set.
    pub fn charset(&self) -> Option<&'a str> {
        self.get("Content-Type").and_then(|x| x.split("charset=").skip(1).next())
    }
}

impl<'a> Deref for MetadataMap<'a> {
    type Target = HashMap<&'a str, &'a str>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for MetadataMap<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn parse_metadata(blob: &str) -> Result<MetadataMap, Error> {
    let mut map = MetadataMap(HashMap::new());
    for line in blob.split('\n').filter(|s| s != &"") {
        let pos = match line.bytes().position(|b| b == b':') {
            Some(p) => p,
            None => return Err(MalformedMetadata),
        };
        map.insert(line[..pos].trim(), line[pos + 1..].trim());
    }
    Ok(map)
}

#[test]
fn test_metadatamap_charset() {
    {
        let mut map = MetadataMap(HashMap::new());
        assert!(map.charset().is_none());
        map.insert("Content-Type", "");
        assert!(map.charset().is_none());
        map.insert("Content-Type", "abc");
        assert!(map.charset().is_none());
        map.insert("Content-Type", "text/plain; charset=utf-42");
        assert_eq!(map.charset().unwrap(), "utf-42");
    }
}
