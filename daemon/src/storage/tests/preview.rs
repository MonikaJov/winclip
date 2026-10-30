// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn preview_is_first_200_code_points() {
    let mut s = storage();
    let text = format!("{}{}", "😀".repeat(150), "ä".repeat(100));
    s.add_item(&text, None).unwrap();
    let preview = &s.get_history(0).unwrap()[0].preview;
    assert_eq!(preview.chars().count(), 200);
    assert_eq!(*preview, format!("{}{}", "😀".repeat(150), "ä".repeat(50)));
}

#[test]
fn get_item_returns_text_beyond_preview() {
    let mut s = storage();
    let text = "x".repeat(500);
    let id = s.add_item(&text, None).unwrap();
    assert_eq!(s.get_item(id).unwrap().text, text);
}
