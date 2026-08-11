use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use super::detector::EnginePaths;
use super::error::EngineError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub duration: Option<u64>,
    pub uploader: Option<String>,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YtDlpDumpJson {
    id: String,
    title: String,
    duration: Option<u64>,
    uploader: Option<String>,
    thumbnail: Option<String>,
    description: Option<String>,
}

pub async fn inspect_url(url: &str, engine: &EnginePaths) -> Result<VideoInfo, EngineError> {
    let mut cmd = Command::new(&engine.yt_dlp);
    let mut args = vec![
        "--dump-json".to_string(),
        "--no-playlist".to_string(),
        "--skip-download".to_string(),
    ];
    if std::env::var("SIPHONIX_DEV_INSECURE_SSL").is_ok() {
        args.push("--no-check-certificates".to_string());
    }
    args.push(url.to_string());

    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await.map_err(|e| EngineError::ProcessFailed {
        code: None,
        stderr: e.to_string(),
    })?;

    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        return Err(EngineError::classify_from_stderr(&stderr_text));
    }

    let stdout_text = String::from_utf8_lossy(&output.stdout);
    let parsed: YtDlpDumpJson = serde_json::from_str(&stdout_text).map_err(|e| {
        EngineError::OutputFileError(format!("Failed to parse metadata JSON: {}", e))
    })?;

    Ok(VideoInfo {
        id: parsed.id,
        title: parsed.title,
        duration: parsed.duration,
        uploader: parsed.uploader,
        thumbnail: parsed.thumbnail,
        description: parsed.description,
    })
}
