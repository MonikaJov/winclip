// SPDX-License-Identifier: GPL-3.0-or-later

use super::schema::SCHEMA_VERSION;

// Never put clipboard contents in a message: errors get logged and sent over D-Bus.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no item with that ID")]
    NotFound,

    #[error("{0}")]
    InvalidArgument(String),

    #[error("item is {size} bytes, the limit is {max} bytes")]
    TooLarge { size: usize, max: u32 },

    #[error("database schema version {0} is newer than this daemon supports ({SCHEMA_VERSION})")]
    UnsupportedSchema(usize),

    #[error("database migration failed: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
