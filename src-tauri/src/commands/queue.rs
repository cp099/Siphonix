use std::sync::Arc;
use tauri::State;
use chrono::Utc;

use crate::engine::builder::DownloadRequest;
use crate::engine::options::DownloadOptions;
use crate::queue::job::DownloadJob;
use crate::queue::scheduler::QueueScheduler;
use crate::queue::state::JobState;

#[tauri::command]
pub async fn enqueue_download(
    scheduler: State<'_, Arc<QueueScheduler>>,
    request: DownloadRequest,
) -> Result<DownloadJob, String> {
    let mut opts = request.options.clone().unwrap_or_else(|| {
        let mut d = DownloadOptions::default();
        d.media_mode = request.media_mode.clone();
        d.output.destination_path = request.destination_path.clone();
        if let Some(ref vf) = request.video_format {
            d.output.container = vf.clone();
        }
        if let Some(ref vq) = request.video_quality {
            d.video.resolution = vq.clone();
        }
        if let Some(ref af) = request.audio_format {
            d.audio.format = af.clone();
        }
        if let Some(ref aq) = request.audio_quality {
            d.audio.quality = aq.clone();
        }
        d
    });

    opts.validate().map_err(|e| e.to_string())?;

    let is_audio = opts.media_mode == "audio";
    let format_str = if is_audio {
        opts.audio.format.clone()
    } else {
        opts.output.container.clone()
    };
    let quality_str = if is_audio {
        opts.audio.quality.clone()
    } else {
        opts.video.resolution.clone()
    };

    let job = DownloadJob {
        id: format!("job-{}-{}", Utc::now().timestamp_millis(), rand_suffix()),
        url: request.url.clone(),
        title: "YouTube Download".to_string(),
        thumbnail_url: None,
        media_mode: opts.media_mode.clone(),
        format: format_str,
        quality: quality_str,
        destination_path: opts.output.destination_path.clone(),
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
        source_video_id: None,
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: opts,
    };

    scheduler.enqueue_job(job).await
}

#[tauri::command]
pub async fn pause_queue(scheduler: State<'_, Arc<QueueScheduler>>) -> Result<(), String> {
    scheduler.set_pause_queue(true).await;
    Ok(())
}

#[tauri::command]
pub async fn resume_queue(scheduler: State<'_, Arc<QueueScheduler>>) -> Result<(), String> {
    scheduler.set_pause_queue(false).await;
    Ok(())
}

#[tauri::command]
pub async fn force_resume_cooldown(scheduler: State<'_, Arc<QueueScheduler>>) -> Result<(), String> {
    scheduler.force_resume_cooldown().await;
    Ok(())
}

#[tauri::command]
pub async fn cancel_job(
    scheduler: State<'_, Arc<QueueScheduler>>,
    job_id: String,
) -> Result<bool, String> {
    Ok(scheduler.cancel_job(&job_id).await)
}

#[tauri::command]
pub async fn set_max_concurrency(
    scheduler: State<'_, Arc<QueueScheduler>>,
    limit: usize,
) -> Result<(), String> {
    scheduler.set_max_concurrency(limit).await;
    Ok(())
}

#[tauri::command]
pub async fn get_queue_jobs(
    scheduler: State<'_, Arc<QueueScheduler>>,
) -> Result<Vec<DownloadJob>, String> {
    Ok(scheduler.get_all_jobs().await)
}

#[tauri::command]
pub async fn get_library_jobs(
    scheduler: State<'_, Arc<QueueScheduler>>,
) -> Result<Vec<DownloadJob>, String> {
    Ok(scheduler.get_library_jobs().await)
}

fn rand_suffix() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new().build_hasher().finish();
    format!("{:x}", s)[..6].to_string()
}
