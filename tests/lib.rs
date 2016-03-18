extern crate encoding;
extern crate gettext;

use encoding::label::encoding_from_whatwg_label;
use gettext::{Catalog, ParseOptions};

use std::fs::File;

#[test]
fn test_integration() {
    let f = File::open("test_cases/integration.mo").unwrap();
    let catalog = Catalog::parse(f).unwrap();

    assert_eq!(catalog.gettext("non-existent"), "non-existent");
    assert_eq!(catalog.gettext("existent"), "egzistuojantis");

    assert_eq!(catalog.ngettext("a bad string", "bad strings", 1),
               "a bad string");
    assert_eq!(catalog.ngettext("a bad string", "bad strings", 2),
               "bad strings");
    assert_eq!(catalog.ngettext("a good string", "good strings", 1),
               "gera eilute");
    assert_eq!(catalog.ngettext("a good string", "good strings", 2),
               "geros eilutes");

    assert_eq!(catalog.pgettext("ctxt", "non-existent"), "non-existent");
    assert_eq!(catalog.pgettext("ctxt", "existent"),
               "egzistuojantis kontekste");

    assert_eq!(catalog.npgettext("ctxt", "a bad string", "bad strings", 1),
               "a bad string");
    assert_eq!(catalog.npgettext("ctxt", "a bad string", "bad strings", 2),
               "bad strings");
    assert_eq!(catalog.npgettext("ctxt", "a good string", "good strings", 1),
               "gera eilute kontekste");
    assert_eq!(catalog.npgettext("ctxt", "a good string", "good strings", 2),
               "geros eilutes kontekste");

}

#[test]
fn test_cp1257() {
    // cp1257_meta
    {
        let reader: &[u8] = include_bytes!("../test_cases/cp1257_meta.mo");
        let catalog = ParseOptions::new().parse(reader).unwrap();
        assert_eq!(catalog.gettext("Garlic"), "Česnakas");
    }

    // cp1257_forced
    {
        let reader: &[u8] = include_bytes!("../test_cases/cp1257_forced.mo");
        for enc_name in &["cp1257", "windows-1257", "x-cp1257"] {
            let encoding = encoding_from_whatwg_label(enc_name).unwrap();
            let catalog = ParseOptions::new().force_encoding(encoding).parse(reader).unwrap();
            assert_eq!(catalog.gettext("Garlic"), "Česnakas");
        }
    }
}
