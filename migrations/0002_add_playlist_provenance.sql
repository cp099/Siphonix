-- Migration 0002: Add YouTube Video ID and Playlist Provenance Columns

ALTER TABLE jobs ADD COLUMN source_video_id TEXT;
ALTER TABLE jobs ADD COLUMN source_playlist_id TEXT;
ALTER TABLE jobs ADD COLUMN source_playlist_title TEXT;
ALTER TABLE jobs ADD COLUMN playlist_entry_index INTEGER;

CREATE INDEX IF NOT EXISTS idx_jobs_video_id ON jobs(source_video_id);
CREATE INDEX IF NOT EXISTS idx_jobs_playlist_id ON jobs(source_playlist_id);
