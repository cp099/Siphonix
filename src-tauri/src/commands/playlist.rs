use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbRepository;
use crate::engine::detector::EngineDetector;
use crate::engine::playlist::{PlaylistEntry, PlaylistInfo, PlaylistInspector};
use crate::queue::job::DownloadJob;
use crate::queue::scheduler::QueueScheduler;
use crate::queue::state::JobState;

#[derive(Debug, Deserialize)]
pub struct EnqueuePlaylistParams {
    pub playlist_id: String,
    pub playlist_title: String,
    pub entries: Vec<PlaylistEntry>,
    pub media_mode: String,
    pub audio_format: Option<String>,
    pub audio_quality: Option<String>,
    pub video_format: Option<String>,
    pub video_quality: Option<String>,
    pub destination_path: String,
}

#[derive(Debug, Serialize)]
pub struct EnqueuePlaylistResult {
    pub added_count: usize,
    pub skipped_count: usize,
}

#[tauri::command]
pub async fn inspect_playlist_url(
    inspector: State<'_, PlaylistInspector>,
    inspection_id: String,
    url: String,
) -> Result<PlaylistInfo, String> {
    let engine = EngineDetector::detect().map_err(|e| e.to_string())?;
    inspector
        .inspect_playlist(&inspection_id, &url, &engine)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_playlist_inspection(
    inspector: State<'_, PlaylistInspector>,
    inspection_id: String,
) -> Result<bool, String> {
    Ok(inspector.cancel_inspection(&inspection_id).await)
}

#[tauri::command]
pub async fn enqueue_playlist_entries(
    scheduler: State<'_, Arc<QueueScheduler>>,
    db: State<'_, Arc<DbRepository>>,
    params: EnqueuePlaylistParams,
) -> Result<EnqueuePlaylistResult, String> {
    let is_audio = params.media_mode == "audio";
    let format_str = if is_audio {
        params.audio_format.unwrap_or_else(|| "MP3".to_string())
    } else {
        params.video_format.unwrap_or_else(|| "MP4".to_string())
    };
    let quality_str = if is_audio {
        params.audio_quality.unwrap_or_else(|| "best".to_string())
    } else {
        params.video_quality.unwrap_or_else(|| "1080p".to_string())
    };

    // Filter available entries
    let available_entries: Vec<&PlaylistEntry> = params
        .entries
        .iter()
        .filter(|e| e.availability == "AVAILABLE" || e.availability == "unlisted" || e.availability == "public")
        .collect();

    let entry_video_ids: Vec<String> = available_entries.iter().map(|e| e.id.clone()).collect();
    let existing_video_ids = db.find_existing_video_ids(&entry_video_ids).await.unwrap_or_default();

    let mut added_count = 0;
    let mut skipped_count = 0;

    for entry in available_entries {
        if existing_video_ids.contains(&entry.id) {
            skipped_count += 1;
            continue;
        }

        let job = DownloadJob {
            id: format!("job-{}-{}", Utc::now().timestamp_millis(), entry.index),
            url: entry.url.clone(),
            title: entry.title.clone(),
            thumbnail_url: entry.thumbnail_url.clone(),
            media_mode: params.media_mode.clone(),
            format: format_str.clone(),
            quality: quality_str.clone(),
            destination_path: params.destination_path.clone(),
            state: JobState::QUEUED,
            progress: 0.0,
            download_speed: None,
            eta: None,
            file_size: None,
            error_message: None,
            last_error_category: None,
            retry_count: 0,
            max_retries: 5,
            next_retry_at: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            source_video_id: Some(entry.id.clone()),
            source_playlist_id: Some(params.playlist_id.clone()),
            source_playlist_title: Some(params.playlist_title.clone()),
            playlist_entry_index: Some(entry.index as u32),
        };

        scheduler.enqueue_job(job).await.map_err(|e| e.to_string())?;
        added_count += 1;
    }

    Ok(EnqueuePlaylistResult {
        added_count,
        skipped_count,
    })
}
