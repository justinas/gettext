//! The `gettext` crate provides functionality for
//! parsing and using Gettext catalogs stored in MO files.

// https://pascalhertleif.de/artikel/good-practices-for-writing-rust-libraries/
#![deny(missing_docs, missing_debug_implementations,
        trivial_casts, trivial_numeric_casts, unused_import_braces)]

mod parser;

use std::collections::HashMap;
use std::ops::Deref;

pub use parser::Error;

/// Returns the number of the appropriate plural form
/// for the given count `n` of objects for germanic languages.
fn plural_form(n: usize) -> usize {
    if n != 1 {
        1
    } else {
        0
    }
}

fn key_with_context(context: &str, key: &str) -> String {
    let mut result = context.to_owned();
    result.push('\x04');
    result.push_str(key);
    result
}

/// Catalog represents a set of translation strings
/// parsed out of one MO file.
#[derive(Debug)]
pub struct Catalog {
    strings: HashMap<String, Message>,
}

impl Catalog {
    fn new() -> Self {
        Catalog { strings: HashMap::new() }
    }

    fn insert(&mut self, msg: Message) {
        let key = match msg.context {
            Some(ref ctxt) => key_with_context(ctxt, &msg.id),
            None => msg.id.clone(),
        };
        self.strings.insert(key, msg);
    }

    /// Returns the singular translation of `msg_id` from the given catalog
    /// or `msg_id` itself if a translation does not exist.
    pub fn gettext<'a>(&'a self, msg_id: &'a str) -> &'a str {
        self.strings.get(&msg_id.to_owned()).and_then(|msg| msg.get_translated(0)).unwrap_or(msg_id)
    }

    /// Returns the plural translation of `msg_id` from the given catalog
    /// with the correct plural form for the number `n` of objects.
    /// Returns msg_id if a translation does not exist and `n == 1`,
    /// msg_id_plural otherwise.
    ///
    /// Currently, the only supported plural formula is `n != 1`.
    pub fn ngettext<'a>(&'a self, msg_id: &'a str, msg_id_plural: &'a str, n: usize) -> &'a str {
        let form_no = plural_form(n);

        match self.strings.get(&msg_id.to_owned()) {
            Some(msg) => {
                msg.get_translated(form_no).unwrap_or_else(|| [msg_id, msg_id_plural][form_no])
            }
            None if form_no == 0 => msg_id,
            None if form_no == 1 => msg_id_plural,
            _ => unreachable!(),
        }
    }

    /// Returns the singular translation of `msg_id`
    /// in the context `msg_context`
    /// or `msg_id` itself if a translation does not exist.
    // TODO: DRY gettext/pgettext
    pub fn pgettext<'a>(&'a self, msg_context: &'a str, msg_id: &'a str) -> &'a str {
        let key = key_with_context(msg_context, &msg_id);
        self.strings.get(&key).and_then(|msg| msg.get_translated(0)).unwrap_or(msg_id)
    }

    /// Returns the plural translation of `msg_id`
    /// in the context `msg_context`
    /// with the correct plural form for the number `n` of objects.
    /// Returns msg_id if a translation does not exist and `n == 1`,
    /// msg_id_plural otherwise.
    ///
    /// Currently, the only supported plural formula is `n != 1`.
    // TODO: DRY ngettext/npgettext
    pub fn npgettext<'a>(&'a self,
                         msg_context: &'a str,
                         msg_id: &'a str,
                         msg_id_plural: &'a str,
                         n: usize)
                         -> &'a str {
        let key = key_with_context(msg_context, &msg_id);
        let form_no = plural_form(n);
        match self.strings.get(&key) {
            Some(msg) => {
                msg.get_translated(form_no).unwrap_or_else(|| [msg_id, msg_id_plural][form_no])
            }
            None if form_no == 0 => msg_id,
            None if form_no == 1 => msg_id_plural,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Message {
    id: String,
    context: Option<String>,
    translated: Vec<String>,
}

impl Message {
    fn new<T: Into<String>>(id: T, context: Option<T>, translated: Vec<T>) -> Self {
        Message {
            id: id.into(),
            context: context.map(Into::into),
            translated: translated.into_iter().map(Into::into).collect(),
        }
    }

    fn get_translated(&self, form_no: usize) -> Option<&str> {
        self.translated.get(form_no).map(|s| s.deref())
    }
}

#[test]
fn catalog_insert() {
    let mut cat = Catalog::new();
    cat.insert(Message::new("thisisid", None, vec![]));
    cat.insert(Message::new("anotherid", Some("context"), vec![]));
    let mut keys = cat.strings.keys().collect::<Vec<_>>();
    keys.sort();
    assert_eq!(keys, &["context\x04anotherid", "thisisid"])
}

#[test]
fn catalog_gettext() {
    let mut cat = Catalog::new();
    cat.insert(Message::new("Text", None, vec!["Tekstas"]));
    cat.insert(Message::new("Image", Some("context"), vec!["Paveikslelis"]));
    assert_eq!(cat.gettext("Text"), "Tekstas");
    assert_eq!(cat.gettext("Image"), "Image");
}

#[test]
fn catalog_ngettext() {
    let mut cat = Catalog::new();
    {
        // n == 1, no translation
        assert_eq!(cat.ngettext("Text", "Texts", 1), "Text");
        // n != 1, no translation
        assert_eq!(cat.ngettext("Text", "Texts", 0), "Texts");
        assert_eq!(cat.ngettext("Text", "Texts", 2), "Texts");
    }
    {
        cat.insert(Message::new("Text", None, vec!["Tekstas", "Tekstai"]));
        // n == 1, translation available
        assert_eq!(cat.ngettext("Text", "Texts", 1), "Tekstas");
        // n != 1, translation available
        assert_eq!(cat.ngettext("Text", "Texts", 0), "Tekstai");
        assert_eq!(cat.ngettext("Text", "Texts", 2), "Tekstai");
    }
}

#[test]
fn catalog_pgettext() {
    let mut cat = Catalog::new();
    cat.insert(Message::new("Text", Some("unit test"), vec!["Tekstas"]));
    assert_eq!(cat.pgettext("unit test", "Text"), "Tekstas");
    assert_eq!(cat.pgettext("integration test", "Text"), "Text");
}

#[test]
fn catalog_npgettext() {
    let mut cat = Catalog::new();
    cat.insert(Message::new("Text", Some("unit test"), vec!["Tekstas", "Tekstai"]));

    assert_eq!(cat.npgettext("unit test", "Text", "Texts", 1), "Tekstas");
    assert_eq!(cat.npgettext("unit test", "Text", "Texts", 0), "Tekstai");
    assert_eq!(cat.npgettext("unit test", "Text", "Texts", 2), "Tekstai");

    assert_eq!(cat.npgettext("integration test", "Text", "Texts", 1),
               "Text");
    assert_eq!(cat.npgettext("integration test", "Text", "Texts", 0),
               "Texts");
    assert_eq!(cat.npgettext("integration test", "Text", "Texts", 2),
               "Texts");
}
