// SPDX-License-Identifier: GPL-3.0-or-later

mod error;
mod files;
mod limits;
mod schema;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

pub use error::{Error, Result};
use limits::{MAX_AGE_DAYS, MAX_ITEM_BYTES, MAX_ITEMS};

pub const PREVIEW_CHARS: u32 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSummary {
    pub id: u64,
    pub preview: String,
    pub has_html: bool,
    pub copied_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub text: String,
    pub html: Option<String>,
}

type Clock = Box<dyn Fn() -> i64 + Send>;

fn system_clock() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

pub struct Storage {
    conn: Connection,
    now: Clock,
}

impl Storage {
    pub fn default_dir() -> Result<PathBuf> {
        files::default_dir()
    }

    pub fn open(dir: &Path) -> Result<Self> {
        let db_path = files::prepare(dir)?;
        Self::init(Connection::open(db_path)?, Box::new(system_clock))
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, Box::new(system_clock))
    }

    fn init(mut conn: Connection, now: Clock) -> Result<Self> {
        schema::setup(&mut conn)?;
        limits::trim(&conn, now())?;
        Ok(Storage { conn, now })
    }

    pub fn add_item(&mut self, text: &str, html: Option<&str>) -> Result<u64> {
        if text.is_empty() {
            return Err(Error::InvalidArgument("text must not be empty".into()));
        }
        let html = html.filter(|h| !h.is_empty());

        let max = limits::get(&self.conn, &MAX_ITEM_BYTES)?;
        let size = text.len() + html.map_or(0, str::len);
        if size > usize::try_from(max).unwrap_or(usize::MAX) {
            return Err(Error::TooLarge { size, max });
        }

        let hash = Sha256::digest(text.as_bytes());
        let now = (self.now)();
        let tx = self.conn.transaction()?;
        // Ordered by `seq`, not `copied_at`: two copies can happen within one second.
        let id: u64 = tx.query_row(
            "INSERT INTO items (seq, text, html, text_hash, copied_at)
             VALUES ((SELECT COALESCE(MAX(seq), 0) + 1 FROM items), ?1, ?2, ?3, ?4)
             ON CONFLICT (text_hash) DO UPDATE SET
                 seq = excluded.seq,
                 html = excluded.html,
                 copied_at = excluded.copied_at
             RETURNING id",
            params![text, html, hash.as_slice(), now],
            |row| row.get(0),
        )?;
        limits::trim(&tx, now)?;
        tx.commit()?;
        Ok(id)
    }

    pub fn get_history(&self, limit: u32) -> Result<Vec<ItemSummary>> {
        self.query_summaries("", limit)
    }

    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<ItemSummary>> {
        self.query_summaries(query, limit)
    }

    fn query_summaries(&self, query: &str, limit: u32) -> Result<Vec<ItemSummary>> {
        // SQLite treats a negative LIMIT as "no limit".
        let limit = if limit == 0 { -1 } else { i64::from(limit) };
        let mut stmt = self.conn.prepare(
            "SELECT id, substr(text, 1, ?1), html IS NOT NULL, copied_at
             FROM items
             WHERE ?2 = '' OR winclip_contains(text, ?2)
             ORDER BY seq DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![PREVIEW_CHARS, query.to_lowercase(), limit], |row| {
            Ok(ItemSummary {
                id: row.get(0)?,
                preview: row.get(1)?,
                has_html: row.get(2)?,
                copied_at: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn get_item(&self, id: u64) -> Result<Item> {
        let id = sql_id(id)?;
        self.conn
            .query_row("SELECT text, html FROM items WHERE id = ?1", [id], |row| {
                Ok(Item {
                    text: row.get(0)?,
                    html: row.get(1)?,
                })
            })
            .optional()?
            .ok_or(Error::NotFound)
    }

    pub fn delete_item(&mut self, id: u64) -> Result<()> {
        let id = sql_id(id)?;
        match self.conn.execute("DELETE FROM items WHERE id = ?1", [id])? {
            0 => Err(Error::NotFound),
            _ => Ok(()),
        }
    }

    pub fn clear_history(&mut self) -> Result<usize> {
        Ok(self.conn.execute("DELETE FROM items", [])?)
    }

    pub fn trim(&mut self) -> Result<usize> {
        limits::trim(&self.conn, (self.now)())
    }

    pub fn max_items(&self) -> Result<u32> {
        limits::get(&self.conn, &MAX_ITEMS)
    }

    pub fn max_age_days(&self) -> Result<u32> {
        limits::get(&self.conn, &MAX_AGE_DAYS)
    }

    pub fn max_item_bytes(&self) -> Result<u32> {
        limits::get(&self.conn, &MAX_ITEM_BYTES)
    }

    pub fn set_max_items(&mut self, value: u32) -> Result<usize> {
        limits::set(&mut self.conn, &MAX_ITEMS, value, (self.now)())
    }

    pub fn set_max_age_days(&mut self, value: u32) -> Result<usize> {
        limits::set(&mut self.conn, &MAX_AGE_DAYS, value, (self.now)())
    }

    pub fn set_max_item_bytes(&mut self, value: u32) -> Result<usize> {
        limits::set(&mut self.conn, &MAX_ITEM_BYTES, value, (self.now)())
    }
}

fn sql_id(id: u64) -> Result<i64> {
    i64::try_from(id).map_err(|_| Error::NotFound)
}
