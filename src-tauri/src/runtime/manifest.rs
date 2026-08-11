use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeArtifact {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub architecture: String,
    pub expected_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManifest {
    pub manifest_version: String,
    pub yt_dlp: RuntimeArtifact,
    pub ffmpeg: RuntimeArtifact,
}

impl Default for RuntimeManifest {
    fn default() -> Self {
        Self {
            manifest_version: "1.0.0".to_string(),
            yt_dlp: RuntimeArtifact {
                name: "yt-dlp".to_string(),
                version: "2024.03.10".to_string(),
                platform: std::env::consts::OS.to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                expected_sha256: None,
            },
            ffmpeg: RuntimeArtifact {
                name: "ffmpeg".to_string(),
                version: "6.1.1".to_string(),
                platform: std::env::consts::OS.to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                expected_sha256: None,
            },
        }
    }
}
