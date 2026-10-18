// SPDX-License-Identifier: GPL-3.0-or-later

mod adding;
mod dedup;
mod delete;
mod files;
mod limits;
mod preview;
mod schema;
mod search;
mod validation;

use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use rusqlite::Connection;

use super::files::DB_FILE_NAME;
use super::limits::SECONDS_PER_DAY;
use super::schema::{MIGRATIONS, SCHEMA_VERSION};
use super::{Error, Item, ItemSummary, Storage};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

const T0: i64 = 1_700_000_000;

fn storage_with_clock() -> (Storage, Arc<AtomicI64>) {
    let time = Arc::new(AtomicI64::new(T0));
    let clock = Arc::clone(&time);
    let storage = Storage::init(
        Connection::open_in_memory().unwrap(),
        Box::new(move || clock.load(Ordering::SeqCst)),
    )
    .unwrap();
    (storage, time)
}

fn storage() -> Storage {
    storage_with_clock().0
}

fn previews(items: &[ItemSummary]) -> Vec<&str> {
    items.iter().map(|i| i.preview.as_str()).collect()
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}
