// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn recopy_moves_to_top_and_keeps_id() {
    let (mut s, time) = storage_with_clock();
    let first = s.add_item("same", Some("<b>old</b>")).unwrap();
    s.add_item("other", None).unwrap();
    time.store(T0 + 60, Ordering::SeqCst);
    let again = s.add_item("same", Some("<b>new</b>")).unwrap();

    assert_eq!(first, again);
    let history = s.get_history(0).unwrap();
    assert_eq!(previews(&history), ["same", "other"]);
    assert_eq!(history[0].copied_at, T0 + 60);
    assert_eq!(
        s.get_item(first).unwrap().html.as_deref(),
        Some("<b>new</b>")
    );
}

#[test]
fn recopy_without_html_drops_old_html() {
    let mut s = storage();
    let id = s.add_item("same", Some("<b>same</b>")).unwrap();
    s.add_item("same", None).unwrap();
    assert_eq!(s.get_item(id).unwrap().html, None);
}

#[test]
fn recopy_within_same_second_still_moves_to_top() {
    let mut s = storage();
    s.add_item("a", None).unwrap();
    s.add_item("b", None).unwrap();
    s.add_item("a", None).unwrap();
    assert_eq!(previews(&s.get_history(0).unwrap()), ["a", "b"]);
}

#[test]
fn dedup_is_case_sensitive() {
    let mut s = storage();
    s.add_item("Hello", None).unwrap();
    s.add_item("hello", None).unwrap();
    assert_eq!(s.get_history(0).unwrap().len(), 2);
}
