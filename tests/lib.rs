extern crate gettext;

use gettext::Catalog;

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
