use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::process::Command;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::repository::{DbLibraryItem, DbRepository};
use crate::queue::job::DownloadJob;

#[derive(Clone)]
pub struct LibraryService {
    db: Arc<DbRepository>,
}

impl LibraryService {
    pub fn new(db: Arc<DbRepository>) -> Self {
        Self { db }
    }

    /// Resolve actual output file on disk.
    /// Priority 1: Use exact path string captured from yt-dlp `--print after_move:filepath`
    /// Priority 2: Fallback scanning destination directory for matching title/ID with valid media extensions.
    pub fn resolve_output_file(job: &DownloadJob, captured_path: Option<&str>) -> Option<PathBuf> {
        // Priority 1: Exact captured path from yt-dlp
        if let Some(p_str) = captured_path {
            let p = PathBuf::from(p_str);
            if p.exists() && p.is_file() {
                return Some(p);
            }
        }

        // Priority 2: Fallback scan of destination folder
        let dest_dir = Path::new(&job.destination_path);
        if !dest_dir.exists() || !dest_dir.is_dir() {
            return None;
        }

        let valid_exts = ["mp4", "mp3", "mkv", "m4a", "webm", "flac", "wav", "aac", "ogg"];

        if let Ok(entries) = std::fs::read_dir(dest_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let lower_ext = ext.to_lowercase();
                        if valid_exts.contains(&lower_ext.as_str()) {
                            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            // Ensure it's not a temp download file
                            if file_name.ends_with(".part") || file_name.ends_with(".ytdl") {
                                continue;
                            }
                            if file_name.contains(&job.title) || (job.source_video_id.is_some() && file_name.contains(job.source_video_id.as_ref().unwrap())) {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Register a completed job into the library_items SQLite table
    pub async fn register_completed_job(
        &self,
        job: &DownloadJob,
        captured_path: Option<&str>,
    ) -> Result<DbLibraryItem, String> {
        let file_path_buf = Self::resolve_output_file(job, captured_path)
            .ok_or_else(|| format!("Could not resolve valid output file for completed job: {}", job.title))?;

        let file_path_str = file_path_buf.to_string_lossy().to_string();
        let file_name_str = file_path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&job.title)
            .to_string();
        let ext_str = file_path_buf
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or(&job.format)
            .to_lowercase();

        let metadata = std::fs::metadata(&file_path_buf).map_err(|e| format!("Failed to read file metadata: {}", e))?;
        let file_size_bytes = metadata.len() as i64;

        let now = Utc::now().to_rfc3339();
        let item_id = format!("lib-{}", job.id);

        let options_json_str = serde_json::to_string(&job.options).ok();

        let library_item = DbLibraryItem {
            id: item_id,
            job_id: job.id.clone(),
            source_video_id: job.source_video_id.clone(),
            title: job.title.clone(),
            file_path: file_path_str,
            file_name: file_name_str,
            file_extension: ext_str,
            media_mode: job.media_mode.clone(),
            format: job.format.clone(),
            quality: job.quality.clone(),
            file_size_bytes,
            duration_seconds: None,
            thumbnail_url: job.thumbnail_url.clone(),
            source_url: job.url.clone(),
            source_playlist_id: job.source_playlist_id.clone(),
            source_playlist_title: job.source_playlist_title.clone(),
            playlist_entry_index: job.playlist_entry_index.map(|i| i as i64),
            created_at: job.created_at.to_rfc3339(),
            completed_at: now.clone(),
            last_verified_at: now,
            file_status: "AVAILABLE".to_string(),
            options_json: options_json_str,
        };

        self.db.insert_library_item(&library_item).await.map_err(|e| e.to_string())?;

        Ok(library_item)
    }

    /// Query library items with search, filter, and sort
    pub async fn get_library_items(
        &self,
        search: Option<String>,
        filter_mode: Option<String>,
        filter_status: Option<String>,
        sort_by: Option<String>,
    ) -> Result<Vec<DbLibraryItem>, String> {
        self.db
            .get_library_items(search.as_deref(), filter_mode.as_deref(), filter_status.as_deref(), sort_by.as_deref())
            .await
            .map_err(|e| e.to_string())
    }

    /// Verify file existence on disk for all registered items and update file_status in SQLite
    pub async fn verify_library_items(&self) -> Result<Vec<DbLibraryItem>, String> {
        let items = self.db.get_library_items(None, None, None, None).await.map_err(|e| e.to_string())?;

        for item in &items {
            let exists = Path::new(&item.file_path).exists();
            let expected_status = if exists { "AVAILABLE" } else { "MISSING" };

            if item.file_status != expected_status {
                let _ = self.db.update_library_item_status(&item.id, expected_status).await;
            }
        }

        self.db.get_library_items(None, None, None, None).await.map_err(|e| e.to_string())
    }

    /// Open media file using OS default application
    pub async fn open_item(&self, item_id: &str) -> Result<(), String> {
        let item = self
            .db
            .get_library_item_by_id(item_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Library item not found: {}", item_id))?;

        let path = Path::new(&item.file_path);
        if !path.exists() {
            let _ = self.db.update_library_item_status(item_id, "MISSING").await;
            return Err(format!("File does not exist at path: {}", item.file_path));
        }

        opener::open(path).map_err(|e| format!("Failed to open file: {}", e))
    }

    /// Reveal containing folder in macOS Finder / Windows Explorer with file selected
    pub async fn reveal_item(&self, item_id: &str) -> Result<(), String> {
        let item = self
            .db
            .get_library_item_by_id(item_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Library item not found: {}", item_id))?;

        let path = Path::new(&item.file_path);
        if !path.exists() {
            let _ = self.db.update_library_item_status(item_id, "MISSING").await;
            return Err(format!("File does not exist at path: {}", item.file_path));
        }

        #[cfg(target_os = "macos")]
        {
            let res = Command::new("open").args(["-R", &item.file_path]).status();
            if let Ok(st) = res {
                if st.success() {
                    return Ok(());
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let res = Command::new("explorer.exe").args(["/select,", &item.file_path]).status();
            if let Ok(st) = res {
                if st.success() {
                    return Ok(());
                }
            }
        }

        // Fallback open containing parent directory
        if let Some(parent) = path.parent() {
            let _ = opener::open(parent);
            Ok(())
        } else {
            Err("Could not determine parent directory".to_string())
        }
    }

    /// Remove SQLite library record WITHOUT deleting physical media file on disk
    pub async fn remove_item_record(&self, item_id: &str) -> Result<(), String> {
        self.db.delete_library_item(item_id).await.map_err(|e| e.to_string())
    }

    /// Permanently delete physical media file on disk (with path canonicalization security) and remove SQLite record
    pub async fn delete_item_file(&self, item_id: &str) -> Result<(), String> {
        let item = self
            .db
            .get_library_item_by_id(item_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Library item not found: {}", item_id))?;

        let target_path = Path::new(&item.file_path);

        // Security check: If target path exists on disk, canonicalize and ensure it matches stored record
        if target_path.exists() {
            let canonical_stored = std::fs::canonicalize(target_path)
                .map_err(|e| format!("Invalid or unresolvable path: {}", e))?;
            let canonical_target = std::fs::canonicalize(&item.file_path)
                .map_err(|e| format!("Invalid target path: {}", e))?;

            if canonical_stored != canonical_target {
                return Err(format!("Security Violation: Target path {:?} does not match stored canonical path {:?}", canonical_target, canonical_stored));
            }

            std::fs::remove_file(&canonical_stored)
                .map_err(|e| format!("Failed to delete physical file: {}", e))?;
        }

        // Remove record from library_items database
        self.db.delete_library_item(item_id).await.map_err(|e| e.to_string())
    }
}
