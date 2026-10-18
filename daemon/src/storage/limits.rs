// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::RangeInclusive;

use rusqlite::{Connection, OptionalExtension, params};

use super::{Error, Result};

pub(super) const SECONDS_PER_DAY: i64 = 86_400;

pub(super) struct Setting {
    key: &'static str,
    default: u32,
    range: RangeInclusive<u32>,
}

pub(super) const MAX_ITEMS: Setting = Setting {
    key: "MaxItems",
    default: 25,
    range: 1..=1000,
};

pub(super) const MAX_AGE_DAYS: Setting = Setting {
    key: "MaxAgeDays",
    default: 0,
    range: 0..=3650,
};

pub(super) const MAX_ITEM_BYTES: Setting = Setting {
    key: "MaxItemBytes",
    default: 1024 * 1024,
    range: 1024..=10 * 1024 * 1024,
};

pub(super) fn get(conn: &Connection, setting: &Setting) -> Result<u32> {
    let value: Option<u32> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [setting.key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value.unwrap_or(setting.default))
}

pub(super) fn set(conn: &mut Connection, setting: &Setting, value: u32, now: i64) -> Result<usize> {
    if !setting.range.contains(&value) {
        return Err(Error::InvalidArgument(format!(
            "{} must be between {} and {}",
            setting.key,
            setting.range.start(),
            setting.range.end()
        )));
    }
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![setting.key, value],
    )?;
    let removed = trim(&tx, now)?;
    tx.commit()?;
    Ok(removed)
}

pub(super) fn trim(conn: &Connection, now: i64) -> Result<usize> {
    let max_items = get(conn, &MAX_ITEMS)?;
    let max_age_days = get(conn, &MAX_AGE_DAYS)?;
    let max_item_bytes = get(conn, &MAX_ITEM_BYTES)?;

    let mut removed = conn.execute(
        "DELETE FROM items WHERE id NOT IN (SELECT id FROM items ORDER BY seq DESC LIMIT ?1)",
        [max_items],
    )?;
    if max_age_days > 0 {
        let cutoff = now - i64::from(max_age_days) * SECONDS_PER_DAY;
        removed += conn.execute("DELETE FROM items WHERE copied_at < ?1", [cutoff])?;
    }
    // Casting to BLOB makes length() count bytes instead of characters.
    removed += conn.execute(
        "DELETE FROM items
         WHERE length(CAST(text AS BLOB)) + COALESCE(length(CAST(html AS BLOB)), 0) > ?1",
        [max_item_bytes],
    )?;
    Ok(removed)
}
