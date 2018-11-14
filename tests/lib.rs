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

    assert_eq!(
        catalog.ngettext("a bad string", "bad strings", 1),
        "a bad string"
    );
    assert_eq!(
        catalog.ngettext("a bad string", "bad strings", 2),
        "bad strings"
    );
    assert_eq!(
        catalog.ngettext("a good string", "good strings", 1),
        "gera eilute"
    );
    assert_eq!(
        catalog.ngettext("a good string", "good strings", 2),
        "geros eilutes"
    );

    assert_eq!(catalog.pgettext("ctxt", "non-existent"), "non-existent");
    assert_eq!(
        catalog.pgettext("ctxt", "existent"),
        "egzistuojantis kontekste"
    );

    assert_eq!(
        catalog.npgettext("ctxt", "a bad string", "bad strings", 1),
        "a bad string"
    );
    assert_eq!(
        catalog.npgettext("ctxt", "a bad string", "bad strings", 2),
        "bad strings"
    );
    assert_eq!(
        catalog.npgettext("ctxt", "a good string", "good strings", 1),
        "gera eilute kontekste"
    );
    assert_eq!(
        catalog.npgettext("ctxt", "a good string", "good strings", 2),
        "geros eilutes kontekste"
    );
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
            let catalog = ParseOptions::new()
                .force_encoding(encoding)
                .parse(reader)
                .unwrap();
            assert_eq!(catalog.gettext("Garlic"), "Česnakas");
        }
    }
}

#[test]
fn test_lt_plural() {
    fn lithuanian_plural(n: u64) -> usize {
        if (n % 10) == 1 && (n % 100) != 11 {
            0
        } else if ((n % 10) >= 2) && ((n % 100) < 10 || (n % 100) >= 20) {
            1
        } else {
            2
        }
    }

    // lt_plural_forced
    {
        let reader: &[u8] = include_bytes!("../test_cases/lt_plural_forced.mo");
        let cat = ParseOptions::new()
            .force_plural(lithuanian_plural)
            .parse(reader)
            .unwrap();

        assert_eq!(cat.ngettext("Garlic", "Garlics", 0), "Česnakų");
        assert_eq!(cat.ngettext("Garlic", "Garlics", 1), "Česnakas");
        for i in 2..9 {
            assert_eq!(cat.ngettext("Garlic", "Garlics", i), "Česnakai");
        }
        for i in 10..20 {
            assert_eq!(cat.ngettext("Garlic", "Garlics", i), "Česnakų");
        }
        assert_eq!(cat.ngettext("Garlic", "Garlics", 21), "Česnakas");
    }
}

#[test]
fn test_complex_plural() {
    let reader: &[u8] = include_bytes!("../test_cases/complex_plural.mo");
    let cat = ParseOptions::new().parse(reader).unwrap();

    assert_eq!(cat.ngettext("Test", "Tests", 0), "Plural 2");
    assert_eq!(cat.ngettext("Test", "Tests", 1), "Singular");
    assert_eq!(cat.ngettext("Test", "Tests", 2), "Plural 1");
    for i in 3..20 {
        assert_eq!(cat.ngettext("Test", "Tests", i), "Plural 2");
    }
}
