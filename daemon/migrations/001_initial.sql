-- SPDX-License-Identifier: GPL-3.0-or-later

CREATE TABLE items (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    seq       INTEGER NOT NULL,
    text      TEXT NOT NULL,
    html      TEXT,
    text_hash BLOB NOT NULL UNIQUE,
    copied_at INTEGER NOT NULL
);

CREATE INDEX items_by_seq ON items (seq);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value INTEGER NOT NULL
);
