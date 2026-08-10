use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use super::error::EngineError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginePaths {
    pub yt_dlp: PathBuf,
    pub ffmpeg: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub yt_dlp_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub ready: bool,
}

pub struct EngineDetector;

impl EngineDetector {
    /// Discover yt-dlp and ffmpeg binaries.
    /// Order of precedence:
    /// 1. Environment variable override (SIPHONIX_YTDLP_PATH / SIPHONIX_FFMPEG_PATH)
    /// 2. System PATH discovery
    /// 3. Common OS fallback paths (/opt/homebrew/bin, /usr/local/bin)
    pub fn detect() -> Result<EnginePaths, EngineError> {
        let yt_dlp = Self::find_binary("yt-dlp", "SIPHONIX_YTDLP_PATH")
            .ok_or_else(|| EngineError::EngineNotFound { name: "yt-dlp".to_string() })?;

        let ffmpeg = Self::find_binary("ffmpeg", "SIPHONIX_FFMPEG_PATH")
            .ok_or_else(|| EngineError::EngineNotFound { name: "ffmpeg".to_string() })?;

        Ok(EnginePaths { yt_dlp, ffmpeg })
    }

    pub fn get_status() -> EngineStatus {
        let yt_dlp_path = Self::find_binary("yt-dlp", "SIPHONIX_YTDLP_PATH")
            .map(|p| p.to_string_lossy().to_string());
        let ffmpeg_path = Self::find_binary("ffmpeg", "SIPHONIX_FFMPEG_PATH")
            .map(|p| p.to_string_lossy().to_string());
        let ready = yt_dlp_path.is_some() && ffmpeg_path.is_some();

        EngineStatus {
            yt_dlp_path,
            ffmpeg_path,
            ready,
        }
    }

    fn find_binary(binary_name: &str, env_var: &str) -> Option<PathBuf> {
        // 1. Check environment variable override
        if let Ok(env_path) = std::env::var(env_var) {
            let path = PathBuf::from(env_path);
            if path.exists() {
                return Some(path);
            }
        }

        // 2. Search system PATH using `which` or standard path splitting
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(binary_name);
                if candidate.is_file() {
                    return Some(candidate);
                }
                #[cfg(target_os = "windows")]
                {
                    let candidate_exe = dir.join(format!("{}.exe", binary_name));
                    if candidate_exe.is_file() {
                        return Some(candidate_exe);
                    }
                }
            }
        }

        // 3. Fallback OS development paths
        let fallback_paths = [
            format!("/opt/homebrew/bin/{}", binary_name),
            format!("/usr/local/bin/{}", binary_name),
            format!("/usr/bin/{}", binary_name),
        ];

        for fallback in &fallback_paths {
            let path = PathBuf::from(fallback);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}
