// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn delete_removes_only_that_item() {
    let mut s = storage();
    let a = s.add_item("a", None).unwrap();
    s.add_item("b", None).unwrap();
    s.delete_item(a).unwrap();
    assert_eq!(previews(&s.get_history(0).unwrap()), ["b"]);
}

#[test]
fn ids_are_never_reused() {
    let mut s = storage();
    let a = s.add_item("a", None).unwrap();
    s.delete_item(a).unwrap();
    let b = s.add_item("b", None).unwrap();
    assert!(b > a);
}

#[test]
fn clear_removes_everything() {
    let mut s = storage();
    s.add_item("a", None).unwrap();
    s.add_item("b", None).unwrap();
    assert_eq!(s.clear_history().unwrap(), 2);
    assert_eq!(s.get_history(0).unwrap(), Vec::new());
}
