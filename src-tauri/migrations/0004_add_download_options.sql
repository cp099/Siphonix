-- Migration 0004: Add Download Options and Presets Table

ALTER TABLE jobs ADD COLUMN options_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE library_items ADD COLUMN options_json TEXT;

CREATE TABLE IF NOT EXISTS presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_default INTEGER NOT NULL DEFAULT 0,
    options_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_presets_is_default ON presets(is_default);
