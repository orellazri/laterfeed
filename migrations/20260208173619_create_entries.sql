CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    body TEXT,
    source_type INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_entries_url ON entries(url);
