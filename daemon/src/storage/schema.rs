// SPDX-License-Identifier: GPL-3.0-or-later

use rusqlite::Connection;
use rusqlite::functions::FunctionFlags;
use rusqlite_migration::{M, Migrations, SchemaVersion};

use super::{Error, Result};

const MIGRATION_LIST: &[M<'static>] = &[M::up(include_str!("../../migrations/001_initial.sql"))];

pub(super) const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_LIST);
pub(super) const SCHEMA_VERSION: usize = MIGRATION_LIST.len();

pub(super) fn setup(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "secure_delete", true)?;
    register_contains(conn)?;

    if let SchemaVersion::Outside(version) = MIGRATIONS.current_version(conn)? {
        return Err(Error::UnsupportedSchema(version.get()));
    }
    MIGRATIONS.to_latest(conn)?;
    Ok(())
}

// SQLite's LIKE only ignores case for ASCII letters.
fn register_contains(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "winclip_contains",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let haystack: String = ctx.get(0)?;
            let needle_lower: String = ctx.get(1)?;
            Ok(haystack.to_lowercase().contains(&needle_lower))
        },
    )?;
    Ok(())
}
