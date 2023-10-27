use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use super::Error;
use crate::Error::MalformedMetadata;

#[derive(Debug, Clone)]

/// Define a struct called `MetadataMap` that represents a map of metadata.
/// It is a simple wrapper around a `HashMap` with `String` keys and `String` values.
/// This struct is used to store key-value pairs of metadata associated with a translation entry or other data.
pub struct MetadataMap(HashMap<String, String>);

impl MetadataMap {
    /// Returns a string that indicates the character set.
    pub fn charset(&self) -> Option<&str> {
        self.get("Content-Type")
            .and_then(|x| x.split("charset=").nth(1))
    }

    /// Returns the number of different plurals and the boolean
    /// expression to determine the form to use depending on
    /// the number of elements.
    ///
    /// Defaults to `n_plurals = 2` and `plural = n!=1` (as in English).
    pub fn plural_forms(&self) -> (Option<usize>, Option<&str>) {
        self.get("Plural-Forms")
            .map(|f| {
                f.split(';').fold((None, None), |(n_pl, pl), prop| {
                    match prop.chars().position(|c| c == '=') {
                        Some(index) => {
                            let (name, value) = prop.split_at(index);
                            let value = value[1..value.len()].trim();
                            match name.trim() {
                                "n_plurals" => (usize::from_str_radix(value, 10).ok(), pl),
                                "plural" => (n_pl, Some(value)),
                                _ => (n_pl, pl),
                            }
                        }
                        None => (n_pl, pl),
                    }
                })
            })
            .unwrap_or((None, None))
    }
}

impl Deref for MetadataMap {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MetadataMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Parses the given metadata blob into a `MetadataMap`.
pub fn parse_metadata(blob: String) -> Result<MetadataMap, Error> {
    let mut map = MetadataMap(HashMap::new());
    for line in blob.split('\n').filter(|s| !s.is_empty()) {
        let pos = match line.bytes().position(|b| b == b':') {
            Some(p) => p,
            None => return Err(MalformedMetadata),
        };
        map.insert(
            line[..pos].trim().to_string(),
            line[pos + 1..].trim().to_string(),
        );
    }
    Ok(map)
}

#[test]
fn test_metadatamap_charset() {
    {
        let mut map = MetadataMap(HashMap::new());
        assert!(map.charset().is_none());
        map.insert("Content-Type".to_string(), "".to_string());
        assert!(map.charset().is_none());
        map.insert("Content-Type".to_string(), "abc".to_string());
        assert!(map.charset().is_none());
        map.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf-42".to_string(),
        );
        assert_eq!(map.charset().unwrap(), "utf-42");
    }
}

#[test]
fn test_metadatamap_plural() {
    {
        let mut map = MetadataMap(HashMap::new());
        assert_eq!(map.plural_forms(), (None, None));

        map.insert("Plural-Forms".to_string(), "".to_string());
        assert_eq!(map.plural_forms(), (None, None));
        // n_plural
        map.insert("Plural-Forms".to_string(), "n_plurals=42".to_string());
        assert_eq!(map.plural_forms(), (Some(42), None));
        // plural is specified
        map.insert(
            "Plural-Forms".to_string(),
            "n_plurals=2; plural=n==12".to_string(),
        );
        assert_eq!(map.plural_forms(), (Some(2), Some("n==12")));
        // plural before n_plurals
        map.insert(
            "Plural-Forms".to_string(),
            "plural=n==12; n_plurals=2".to_string(),
        );
        assert_eq!(map.plural_forms(), (Some(2), Some("n==12")));
        // with spaces
        map.insert(
            "Plural-Forms".to_string(),
            " n_plurals = 42 ; plural = n >  10   ".to_string(),
        );
        assert_eq!(map.plural_forms(), (Some(42), Some("n >  10")));
    }
}
