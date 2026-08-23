CREATE TABLE IF NOT EXISTS library_items (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL,
    source_video_id TEXT,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_extension TEXT NOT NULL,
    media_mode TEXT NOT NULL,
    format TEXT NOT NULL,
    quality TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    duration_seconds INTEGER,
    thumbnail_url TEXT,
    source_url TEXT NOT NULL,
    source_playlist_id TEXT,
    source_playlist_title TEXT,
    playlist_entry_index INTEGER,
    created_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    last_verified_at TEXT NOT NULL,
    file_status TEXT NOT NULL DEFAULT 'AVAILABLE'
);

CREATE INDEX IF NOT EXISTS idx_lib_video_id ON library_items(source_video_id);
CREATE INDEX IF NOT EXISTS idx_lib_status ON library_items(file_status);
CREATE INDEX IF NOT EXISTS idx_lib_created_at ON library_items(created_at);
CREATE INDEX IF NOT EXISTS idx_lib_completed_at ON library_items(completed_at);
CREATE INDEX IF NOT EXISTS idx_lib_ext ON library_items(file_extension);
CREATE INDEX IF NOT EXISTS idx_lib_playlist_id ON library_items(source_playlist_id);
