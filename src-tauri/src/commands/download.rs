use tauri::{State, WebviewWindow};
use crate::engine::{inspect_url, DownloadManager, DownloadRequest, EngineDetector, EngineStatus, VideoInfo};

#[tauri::command]
pub async fn inspect_video_url(url: String) -> Result<VideoInfo, String> {
    let engine = EngineDetector::detect().map_err(|e| e.to_string())?;
    inspect_url(&url, &engine).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_download(
    window: WebviewWindow,
    manager: State<'_, DownloadManager>,
    request: DownloadRequest,
) -> Result<String, String> {
    let job_id = format!("job-{}-{}", chrono_timestamp(), rand_string());

    manager
        .start_download(window, job_id.clone(), request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_id)
}

#[tauri::command]
pub async fn cancel_download(
    manager: State<'_, DownloadManager>,
    job_id: String,
) -> Result<bool, String> {
    Ok(manager.cancel_download(&job_id).await)
}

#[tauri::command]
pub fn get_engine_status() -> EngineStatus {
    EngineDetector::get_status()
}

fn chrono_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn rand_string() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new().build_hasher().finish();
    format!("{:x}", s)[..6].to_string()
}
