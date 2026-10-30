// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn new_files_are_private() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("winclip");
    Storage::open(&dir).unwrap();
    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&dir.join(DB_FILE_NAME)), 0o600);
}

#[test]
fn loose_permissions_are_fixed_on_open() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("winclip");
    Storage::open(&dir).unwrap();
    fs::set_permissions(&dir, Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(dir.join(DB_FILE_NAME), Permissions::from_mode(0o644)).unwrap();

    Storage::open(&dir).unwrap();
    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&dir.join(DB_FILE_NAME)), 0o600);
}

#[test]
fn history_and_settings_survive_reopen() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("winclip");
    {
        let mut s = Storage::open(&dir).unwrap();
        s.set_max_items(5).unwrap();
        s.add_item("persisted", Some("<p>persisted</p>")).unwrap();
    }
    let s = Storage::open(&dir).unwrap();
    assert_eq!(s.max_items().unwrap(), 5);
    assert_eq!(previews(&s.get_history(0).unwrap()), ["persisted"]);
}
