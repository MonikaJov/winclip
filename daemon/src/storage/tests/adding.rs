// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn history_is_newest_first() {
    let mut s = storage();
    s.add_item("one", None).unwrap();
    s.add_item("two", None).unwrap();
    s.add_item("three", None).unwrap();
    assert_eq!(
        previews(&s.get_history(0).unwrap()),
        ["three", "two", "one"]
    );
}

#[test]
fn limit_zero_returns_all_and_limit_caps() {
    let mut s = storage();
    for t in ["a", "b", "c"] {
        s.add_item(t, None).unwrap();
    }
    assert_eq!(s.get_history(0).unwrap().len(), 3);
    assert_eq!(previews(&s.get_history(2).unwrap()), ["c", "b"]);
}

#[test]
fn html_is_kept_alongside_text() {
    let mut s = storage();
    let rich = s.add_item("rich", Some("<b>rich</b>")).unwrap();
    let plain = s.add_item("plain", None).unwrap();
    let empty_html = s.add_item("empty html", Some("")).unwrap();

    let history = s.get_history(0).unwrap();
    assert!(!history[0].has_html);
    assert!(!history[1].has_html);
    assert!(history[2].has_html);

    assert_eq!(
        s.get_item(rich).unwrap(),
        Item {
            text: "rich".into(),
            html: Some("<b>rich</b>".into()),
        }
    );
    assert_eq!(s.get_item(plain).unwrap().html, None);
    assert_eq!(s.get_item(empty_html).unwrap().html, None);
}
