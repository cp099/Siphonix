use serde::{Deserialize, Serialize};
use super::resolver::{EngineSource, ResolvedEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EngineStatusState {
    Ready,
    Missing,
    Outdated,
    Incompatible,
    Corrupted,
    Checking,
}

impl EngineStatusState {
    pub fn as_str(&self) -> &'static str {
        match self {
            EngineStatusState::Ready => "READY",
            EngineStatusState::Missing => "MISSING",
            EngineStatusState::Outdated => "OUTDATED",
            EngineStatusState::Incompatible => "INCOMPATIBLE",
            EngineStatusState::Corrupted => "CORRUPTED",
            EngineStatusState::Checking => "CHECKING",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub name: String,
    pub path: Option<String>,
    pub version: Option<String>,
    pub source: EngineSource,
    pub compatible: bool,
    pub status: EngineStatusState,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub level: String, // "info" | "warning" | "error"
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub ready: bool,
    pub yt_dlp: EngineInfo,
    pub ffmpeg: EngineInfo,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct HealthEvaluator;

impl HealthEvaluator {
    pub fn evaluate(
        resolved_ytdlp: &ResolvedEngine,
        resolved_ffmpeg: &ResolvedEngine,
    ) -> RuntimeStatus {
        let yt_info = Self::build_engine_info(resolved_ytdlp);
        let ffmpeg_info = Self::build_engine_info(resolved_ffmpeg);

        let mut diagnostics = Vec::new();

        if yt_info.status != EngineStatusState::Ready {
            diagnostics.push(Diagnostic {
                code: "YTDLP_UNAVAILABLE".to_string(),
                level: "error".to_string(),
                message: format!("yt-dlp is unavailable or incompatible ({})", yt_info.error.as_deref().unwrap_or("Missing")),
            });
        } else {
            diagnostics.push(Diagnostic {
                code: "YTDLP_READY".to_string(),
                level: "info".to_string(),
                message: format!("yt-dlp version {} is ready ({:?})", yt_info.version.as_deref().unwrap_or(""), yt_info.source),
            });
        }

        if ffmpeg_info.status != EngineStatusState::Ready {
            diagnostics.push(Diagnostic {
                code: "FFMPEG_UNAVAILABLE".to_string(),
                level: "warning".to_string(),
                message: format!("FFmpeg is unavailable ({})", ffmpeg_info.error.as_deref().unwrap_or("Missing")),
            });
        } else {
            diagnostics.push(Diagnostic {
                code: "FFMPEG_READY".to_string(),
                level: "info".to_string(),
                message: format!("FFmpeg version {} is ready ({:?})", ffmpeg_info.version.as_deref().unwrap_or(""), ffmpeg_info.source),
            });
        }

        let ready = yt_info.status == EngineStatusState::Ready && ffmpeg_info.status == EngineStatusState::Ready;

        RuntimeStatus {
            ready,
            yt_dlp: yt_info,
            ffmpeg: ffmpeg_info,
            diagnostics,
        }
    }

    fn build_engine_info(resolved: &ResolvedEngine) -> EngineInfo {
        let status = if resolved.path.is_none() {
            EngineStatusState::Missing
        } else if !resolved.compatible {
            EngineStatusState::Outdated
        } else if resolved.error.is_some() {
            EngineStatusState::Corrupted
        } else {
            EngineStatusState::Ready
        };

        EngineInfo {
            name: resolved.engine_type.name().to_string(),
            path: resolved.path.as_ref().map(|p| p.to_string_lossy().to_string()),
            version: resolved.version.clone(),
            source: resolved.source.clone(),
            compatible: resolved.compatible,
            status,
            error: resolved.error.clone(),
        }
    }
}
