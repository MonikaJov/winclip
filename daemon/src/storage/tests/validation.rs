// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn empty_text_is_rejected() {
    let mut s = storage();
    assert!(matches!(
        s.add_item("", None),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        s.add_item("", Some("<b>x</b>")),
        Err(Error::InvalidArgument(_))
    ));
    assert_eq!(s.get_history(0).unwrap(), Vec::new());
}

#[test]
fn size_limit_counts_bytes_of_text_plus_html() {
    let mut s = storage();
    s.set_max_item_bytes(1024).unwrap();
    let text = "é".repeat(500);

    assert!(s.add_item(&text, Some(&"h".repeat(24))).is_ok());
    assert!(matches!(
        s.add_item(&text, Some(&"h".repeat(25))),
        Err(Error::TooLarge { size: 1025, .. })
    ));
}

#[test]
fn unknown_ids_are_not_found() {
    let mut s = storage();
    assert!(matches!(s.get_item(1), Err(Error::NotFound)));
    assert!(matches!(s.get_item(u64::MAX), Err(Error::NotFound)));
    assert!(matches!(s.delete_item(1), Err(Error::NotFound)));
    assert!(matches!(s.delete_item(u64::MAX), Err(Error::NotFound)));
}

#[test]
fn errors_do_not_contain_clipboard_contents() {
    let mut s = storage();
    s.set_max_item_bytes(1024).unwrap();
    let secret = "hunter2".repeat(200);
    let err = s.add_item(&secret, None).unwrap_err();
    assert!(!err.to_string().contains("hunter2"));
}
