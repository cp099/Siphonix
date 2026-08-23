use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub managed_dir: PathBuf,
}

impl RuntimePaths {
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            managed_dir: app_data_dir.join("runtime"),
        }
    }

    pub fn managed_ytdlp_dir(&self) -> PathBuf {
        self.managed_dir.join("yt-dlp")
    }

    pub fn managed_ffmpeg_dir(&self) -> PathBuf {
        self.managed_dir.join("ffmpeg")
    }

    pub fn managed_ytdlp_binary(&self) -> PathBuf {
        let binary_name = if cfg!(target_os = "windows") {
            "yt-dlp.exe"
        } else {
            "yt-dlp"
        };
        self.managed_ytdlp_dir().join(binary_name)
    }

    pub fn managed_ffmpeg_binary(&self) -> PathBuf {
        let binary_name = if cfg!(target_os = "windows") {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        };
        self.managed_ffmpeg_dir().join(binary_name)
    }

    pub fn is_within_managed_directory(&self, path: &Path) -> bool {
        path.starts_with(&self.managed_dir)
    }
}
