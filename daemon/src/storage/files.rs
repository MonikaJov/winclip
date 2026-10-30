// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, DirBuilder, OpenOptions, Permissions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::Result;

pub(super) const DB_FILE_NAME: &str = "history.db";
const DIR_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

pub(super) fn default_dir() -> Result<PathBuf> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join(".local/share"))
                .filter(|p| p.is_absolute())
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "neither XDG_DATA_HOME nor HOME is set to an absolute path",
            )
        })?;
    Ok(data_home.join("winclip"))
}

pub(super) fn prepare(dir: &Path) -> Result<PathBuf> {
    DirBuilder::new()
        .recursive(true)
        .mode(DIR_MODE)
        .create(dir)?;
    fs::set_permissions(dir, Permissions::from_mode(DIR_MODE))?;

    // Create the file ourselves so it never exists with SQLite's default mode.
    let db_path = dir.join(DB_FILE_NAME);
    OpenOptions::new()
        .append(true)
        .create(true)
        .mode(FILE_MODE)
        .open(&db_path)?;
    fs::set_permissions(&db_path, Permissions::from_mode(FILE_MODE))?;

    Ok(db_path)
}
