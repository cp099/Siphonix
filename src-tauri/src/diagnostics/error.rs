use serde::{Deserialize, Serialize};
use crate::engine::error::EngineError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedDiagnosticFailure {
    pub category: String,
    pub human_explanation: String,
    pub next_action: String,
    pub technical_details: String,
    pub is_retryable: bool,
}

pub struct DiagnosticErrorClassifier;

impl DiagnosticErrorClassifier {
    pub fn classify_engine_error(err: &EngineError) -> ClassifiedDiagnosticFailure {
        match err {
            EngineError::RateLimited(msg) => ClassifiedDiagnosticFailure {
                category: "RATE_LIMITED".to_string(),
                human_explanation: "YouTube temporary rate limit reached. Download paused.".to_string(),
                next_action: "Siphonix will automatically resume when the cooldown expires.".to_string(),
                technical_details: msg.clone(),
                is_retryable: true,
            },
            EngineError::VideoUnavailable(reason) => ClassifiedDiagnosticFailure {
                category: "VIDEO_UNAVAILABLE".to_string(),
                human_explanation: "This video is unavailable or has been removed from YouTube.".to_string(),
                next_action: "Verify the video URL in a web browser.".to_string(),
                technical_details: reason.clone(),
                is_retryable: false,
            },
            EngineError::AuthenticationRequired(reason) => ClassifiedDiagnosticFailure {
                category: "AUTH_REQUIRED".to_string(),
                human_explanation: "This video requires age verification or sign-in.".to_string(),
                next_action: "Age-restricted content is currently not supported without credentials.".to_string(),
                technical_details: reason.clone(),
                is_retryable: false,
            },
            EngineError::EngineNotFound { name } => ClassifiedDiagnosticFailure {
                category: "ENGINE_MISSING".to_string(),
                human_explanation: format!("The download engine executable ({}) is unavailable.", name),
                next_action: "Go to Settings → Runtime to check runtime diagnostics.".to_string(),
                technical_details: err.to_string(),
                is_retryable: false,
            },
            EngineError::TemporaryNetworkError(reason) => ClassifiedDiagnosticFailure {
                category: "NETWORK_FAILURE".to_string(),
                human_explanation: "A network error occurred while connecting to YouTube.".to_string(),
                next_action: "Check your internet connection and click Retry.".to_string(),
                technical_details: reason.clone(),
                is_retryable: true,
            },
            EngineError::FFmpegProcessingError(reason) => ClassifiedDiagnosticFailure {
                category: "FFMPEG_FAILURE".to_string(),
                human_explanation: "FFmpeg failed while processing audio/video conversion.".to_string(),
                next_action: "Check Settings → Runtime to verify FFmpeg installation.".to_string(),
                technical_details: reason.clone(),
                is_retryable: true,
            },
            EngineError::PermissionDenied(reason) => ClassifiedDiagnosticFailure {
                category: "PERMISSION_DENIED".to_string(),
                human_explanation: "Permission denied when attempting to write the output file.".to_string(),
                next_action: "Check destination folder write permissions.".to_string(),
                technical_details: reason.clone(),
                is_retryable: false,
            },
            EngineError::DiskFull(reason) => ClassifiedDiagnosticFailure {
                category: "DISK_FULL".to_string(),
                human_explanation: "Not enough disk space available for download.".to_string(),
                next_action: "Free up storage space on the target drive.".to_string(),
                technical_details: reason.clone(),
                is_retryable: false,
            },
            EngineError::Cancelled => ClassifiedDiagnosticFailure {
                category: "CANCELLED".to_string(),
                human_explanation: "Download job was cancelled by user request.".to_string(),
                next_action: "Re-queue the job if you wish to download it later.".to_string(),
                technical_details: "Job cancelled".to_string(),
                is_retryable: false,
            },
            EngineError::ProcessFailed { stderr, code } => {
                if stderr.to_lowercase().contains("ffmpeg") {
                    ClassifiedDiagnosticFailure {
                        category: "FFMPEG_FAILURE".to_string(),
                        human_explanation: "FFmpeg failed while processing audio/video conversion.".to_string(),
                        next_action: "Check Settings → Runtime to verify FFmpeg installation.".to_string(),
                        technical_details: format!("Exit Code: {:?}\nStderr: {}", code, stderr),
                        is_retryable: true,
                    }
                } else {
                    ClassifiedDiagnosticFailure {
                        category: "DOWNLOAD_FAILED".to_string(),
                        human_explanation: "yt-dlp encountered an error during download execution.".to_string(),
                        next_action: "Click Retry to attempt downloading again.".to_string(),
                        technical_details: format!("Exit Code: {:?}\nStderr: {}", code, stderr),
                        is_retryable: true,
                    }
                }
            }
            _ => ClassifiedDiagnosticFailure {
                category: "GENERAL_FAILURE".to_string(),
                human_explanation: "An unexpected error occurred during download execution.".to_string(),
                next_action: "Click Retry or view technical log details in Settings.".to_string(),
                technical_details: err.to_string(),
                is_retryable: true,
            },
        }
    }
}
