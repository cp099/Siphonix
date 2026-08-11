use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use super::detector::EnginePaths;
use super::error::EngineError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistEntry {
    pub id: String,
    pub index: usize, // 1-based original position
    pub url: String,
    pub title: String,
    pub duration: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub availability: String, // "AVAILABLE", "UNAVAILABLE", "PRIVATE", "DELETED", "UNKNOWN"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistInfo {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub webpage_url: Option<String>,
    pub entry_count: usize,
    pub available_count: usize,
    pub entries: Vec<PlaylistEntry>,
}

#[derive(Debug, Deserialize)]
struct YtDlpRawEntry {
    id: Option<String>,
    title: Option<String>,
    duration: Option<f64>,
    url: Option<String>,
    webpage_url: Option<String>,
    thumbnail: Option<String>,
    availability: Option<String>,
    _type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YtDlpRawPlaylist {
    id: Option<String>,
    title: Option<String>,
    uploader: Option<String>,
    webpage_url: Option<String>,
    entries: Option<Vec<YtDlpRawEntry>>,
}

#[derive(Clone, Default)]
pub struct PlaylistInspector {
    active_inspections: Arc<Mutex<HashMap<String, Child>>>,
}

impl PlaylistInspector {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn inspect_playlist(&self, inspection_id: &str, url: &str, engine: &EnginePaths) -> Result<PlaylistInfo, EngineError> {
        let mut cmd = Command::new(&engine.yt_dlp);
        let candidate_ca_paths = [
            "/opt/homebrew/lib/python3.14/site-packages/certifi/cacert.pem",
            "/opt/homebrew/lib/python3.12/site-packages/certifi/cacert.pem",
            "/etc/ssl/certs/ca-certificates.crt",
        ];
        if std::env::var("SSL_CERT_FILE").is_err() {
            for path in candidate_ca_paths {
                if std::path::Path::new(path).exists() {
                    cmd.env("SSL_CERT_FILE", path);
                    cmd.env("REQUESTS_CA_BUNDLE", path);
                    break;
                }
            }
        }

        let mut args = vec![
            "--flat-playlist".to_string(),
            "--dump-single-json".to_string(),
            "--skip-download".to_string(),
        ];
        if std::env::var("SIPHONIX_DEV_INSECURE_SSL").is_ok() {
            args.push("--no-check-certificates".to_string());
        }
        args.push(url.to_string());

        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| EngineError::ProcessFailed {
            code: None,
            stderr: e.to_string(),
        })?;

        {
            let mut guard = self.active_inspections.lock().await;
            guard.insert(inspection_id.to_string(), child);
        }

        // Wait for child process output
        let guard_remove = {
            let mut guard = self.active_inspections.lock().await;
            guard.remove(inspection_id)
        };

        let output = if let Some(mut c) = guard_remove {
            c.wait_with_output().await.map_err(|e| EngineError::ProcessFailed {
                code: None,
                stderr: e.to_string(),
            })?
        } else {
            return Err(EngineError::Cancelled);
        };

        if !output.status.success() {
            let stderr_text = String::from_utf8_lossy(&output.stderr);
            return Err(EngineError::classify_from_stderr(&stderr_text));
        }

        let stdout_text = String::from_utf8_lossy(&output.stdout);
        let raw: YtDlpRawPlaylist = serde_json::from_str(&stdout_text).map_err(|e| {
            EngineError::OutputFileError(format!("Failed to parse playlist JSON: {}", e))
        })?;

        let playlist_id = raw.id.unwrap_or_else(|| "playlist".to_string());
        let playlist_title = raw.title.unwrap_or_else(|| "YouTube Playlist".to_string());

        let raw_entries = raw.entries.unwrap_or_default();
        let mut entries = Vec::new();
        let mut available_count = 0;

        for (idx, entry) in raw_entries.into_iter().enumerate() {
            let one_based_index = idx + 1;
            let v_id = entry.id.unwrap_or_else(|| format!("deleted-{}", one_based_index));
            let title = entry.title.unwrap_or_else(|| format!("[Deleted video #{}]", one_based_index));
            let v_url = entry.webpage_url.or(entry.url).unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", v_id));

            let availability = if title.contains("[Deleted video]") || entry._type.as_deref() == Some("deleted") {
                "DELETED".to_string()
            } else if title.contains("[Private video]") || entry._type.as_deref() == Some("private") {
                "PRIVATE".to_string()
            } else if entry.availability.as_deref() == Some("unlisted") || entry.availability.as_deref() == Some("public") {
                available_count += 1;
                "AVAILABLE".to_string()
            } else if entry.availability.is_some() {
                entry.availability.unwrap()
            } else {
                available_count += 1;
                "AVAILABLE".to_string()
            };

            entries.push(PlaylistEntry {
                id: v_id,
                index: one_based_index,
                url: v_url,
                title,
                duration: entry.duration.map(|d| d as u64),
                thumbnail_url: entry.thumbnail,
                availability,
            });
        }

        Ok(PlaylistInfo {
            id: playlist_id,
            title: playlist_title,
            uploader: raw.uploader,
            webpage_url: raw.webpage_url,
            entry_count: entries.len(),
            available_count,
            entries,
        })
    }

    pub async fn cancel_inspection(&self, inspection_id: &str) -> bool {
        let mut guard = self.active_inspections.lock().await;
        if let Some(mut child) = guard.remove(inspection_id) {
            let _ = child.start_kill();
            true
        } else {
            false
        }
    }
}
