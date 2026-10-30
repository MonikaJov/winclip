// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn search_is_case_insensitive_including_non_ascii() {
    let mut s = storage();
    s.add_item("Grüße aus MÜNCHEN", None).unwrap();
    s.add_item("unrelated", None).unwrap();
    assert_eq!(
        previews(&s.search("münchen", 0).unwrap()),
        ["Grüße aus MÜNCHEN"]
    );
    assert_eq!(
        previews(&s.search("GRÜSSE", 0).unwrap()).len(),
        0,
        "full case folding (ß -> ss) is not supported, only lowercasing"
    );
}

#[test]
fn search_treats_sql_wildcards_literally() {
    let mut s = storage();
    s.add_item("100% sure", None).unwrap();
    s.add_item("1000 items", None).unwrap();
    s.add_item("a_b", None).unwrap();
    s.add_item("axb", None).unwrap();
    assert_eq!(previews(&s.search("0%", 0).unwrap()), ["100% sure"]);
    assert_eq!(previews(&s.search("a_b", 0).unwrap()), ["a_b"]);
}

#[test]
fn search_is_safe_against_sql_injection() {
    let mut s = storage();
    s.add_item("keep me", None).unwrap();
    assert_eq!(s.search("'; DROP TABLE items; --", 0).unwrap(), Vec::new());
    assert_eq!(s.get_history(0).unwrap().len(), 1);
}

#[test]
fn empty_search_returns_history() {
    let mut s = storage();
    for t in ["a", "b", "c"] {
        s.add_item(t, None).unwrap();
    }
    assert_eq!(s.search("", 0).unwrap(), s.get_history(0).unwrap());
}
