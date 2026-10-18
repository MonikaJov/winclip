// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn migrations_are_valid() {
    MIGRATIONS.validate().unwrap();
}

#[test]
fn newer_schema_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("winclip");
    Storage::open(&dir).unwrap();
    Connection::open(dir.join(DB_FILE_NAME))
        .unwrap()
        .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
        .unwrap();
    assert!(matches!(
        Storage::open(&dir),
        Err(Error::UnsupportedSchema(v)) if v == SCHEMA_VERSION + 1
    ));
}
