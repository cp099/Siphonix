use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum EngineError {
    #[error("Engine binary '{name}' was not found on system PATH or configured location")]
    EngineNotFound { name: String },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Video unavailable: {0}")]
    VideoUnavailable(String),

    #[error("Temporary network error: {0}")]
    TemporaryNetworkError(String),

    #[error("Rate limited by source: {0}")]
    RateLimited(String),

    #[error("Authentication required for content: {0}")]
    AuthenticationRequired(String),

    #[error("Requested format unavailable: {0}")]
    FormatUnavailable(String),

    #[error("FFmpeg processing error: {0}")]
    FFmpegProcessingError(String),

    #[error("Permission denied when writing output file: {0}")]
    PermissionDenied(String),

    #[error("Disk full: {0}")]
    DiskFull(String),

    #[error("Output file error: {0}")]
    OutputFileError(String),

    #[error("Download was cancelled by user")]
    Cancelled,

    #[error("Process exited with code {code:?}: {stderr}")]
    ProcessFailed { code: Option<i32>, stderr: String },
}

impl EngineError {
    pub fn classify_from_stderr(stderr: &str) -> Self {
        let stderr_lower = stderr.to_lowercase();

        if stderr_lower.contains("video unavailable")
            || stderr_lower.contains("private video")
            || stderr_lower.contains("has been removed")
            || stderr_lower.contains("this video is not available")
        {
            Self::VideoUnavailable(stderr.to_string())
        } else if stderr_lower.contains("http error 429")
            || stderr_lower.contains("too many requests")
        {
            Self::RateLimited(stderr.to_string())
        } else if stderr_lower.contains("sign in to confirm your age")
            || stderr_lower.contains("members-only")
            || stderr_lower.contains("sign in to view")
            || stderr_lower.contains("requires authentication")
        {
            Self::AuthenticationRequired(stderr.to_string())
        } else if stderr_lower.contains("permission denied")
            || stderr_lower.contains("access is denied")
            || stderr_lower.contains("read-only file system")
            || stderr_lower.contains("operation not permitted")
        {
            Self::PermissionDenied(stderr.to_string())
        } else if stderr_lower.contains("no space left on device")
            || stderr_lower.contains("disk full")
            || stderr_lower.contains("not enough space")
        {
            Self::DiskFull(stderr.to_string())
        } else if stderr_lower.contains("requested format is not available")
            || stderr_lower.contains("no suitable format")
        {
            Self::FormatUnavailable(stderr.to_string())
        } else if stderr_lower.contains("ffmpeg")
            && (stderr_lower.contains("error")
                || stderr_lower.contains("failed")
                || stderr_lower.contains("invalid data found"))
        {
            Self::FFmpegProcessingError(stderr.to_string())
        } else if stderr_lower.contains("unable to download webpage")
            || stderr_lower.contains("connection timed out")
            || stderr_lower.contains("network is unreachable")
            || stderr_lower.contains("connection refused")
            || stderr_lower.contains("connection reset by peer")
            || stderr_lower.contains("temporary failure in name resolution")
            || stderr_lower.contains("http error 500")
            || stderr_lower.contains("http error 502")
            || stderr_lower.contains("http error 503")
            || stderr_lower.contains("http error 504")
            || stderr_lower.contains("ssl: certificate_verify_failed")
            || stderr_lower.contains("tls error")
        {
            Self::TemporaryNetworkError(stderr.to_string())
        } else {
            Self::ProcessFailed {
                code: None,
                stderr: stderr.to_string(),
            }
        }
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            Self::EngineNotFound { name } => {
                format!("Required engine binary '{}' is missing. Please run `siphonix --diagnostics` to verify installation.", name)
            }
            Self::InvalidUrl(msg) => {
                format!("The provided URL is invalid: {}", msg)
            }
            Self::VideoUnavailable(_) => {
                "This video is unavailable, deleted, or marked private by the uploader.".to_string()
            }
            Self::RateLimited(_) => {
                "YouTube temporarily rate-limited downloads from your IP. Siphonix will automatically retry after a cooldown.".to_string()
            }
            Self::AuthenticationRequired(_) => {
                "This video requires authentication (age-restricted or members-only). Public videos can be downloaded directly.".to_string()
            }
            Self::FormatUnavailable(_) => {
                "The requested format or quality resolution is unavailable for this video. Try selecting 'Best Available'.".to_string()
            }
            Self::FFmpegProcessingError(_) => {
                "FFmpeg encountered an error while multiplexing or encoding the media stream.".to_string()
            }
            Self::PermissionDenied(_) => {
                "Permission denied while writing to the destination folder. Please choose a writable directory.".to_string()
            }
            Self::DiskFull(_) => {
                "Not enough disk space available. Please free up disk space in your download folder and retry.".to_string()
            }
            Self::TemporaryNetworkError(_) => {
                "Temporary network connection failure. Please check your internet connection and retry.".to_string()
            }
            Self::OutputFileError(msg) => {
                format!("Output file error: {}", msg)
            }
            Self::Cancelled => {
                "Download was cancelled.".to_string()
            }
            Self::ProcessFailed { code, stderr } => {
                let clean_line = stderr
                    .lines()
                    .rev()
                    .find(|l| l.contains("ERROR:") || l.contains("Error:") || l.contains("fatal:"))
                    .unwrap_or_else(|| stderr.lines().next().unwrap_or("Process failed"));
                format!("Download process failed (exit {:?}): {}", code, clean_line.trim())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_rate_limited() {
        let stderr = "ERROR: [youtube] dQw4w9WgXcQ: HTTP Error 429: Too Many Requests";
        let err = EngineError::classify_from_stderr(stderr);
        match err {
            EngineError::RateLimited(msg) => assert!(msg.contains("HTTP Error 429")),
            _ => panic!("Expected RateLimited variant"),
        }
    }

    #[test]
    fn test_classify_video_unavailable() {
        let stderr = "ERROR: [youtube] dQw4w9WgXcQ: Video unavailable. This video is private.";
        let err = EngineError::classify_from_stderr(stderr);
        match err {
            EngineError::VideoUnavailable(msg) => assert!(msg.contains("Video unavailable")),
            _ => panic!("Expected VideoUnavailable variant"),
        }
    }

    #[test]
    fn test_classify_authentication_required() {
        let stderr = "ERROR: [youtube] dQw4w9WgXcQ: Sign in to confirm your age";
        let err = EngineError::classify_from_stderr(stderr);
        match err {
            EngineError::AuthenticationRequired(msg) => assert!(msg.contains("Sign in")),
            _ => panic!("Expected AuthenticationRequired variant"),
        }
    }

    #[test]
    fn test_classify_disk_full() {
        let stderr = "ERROR: [download] Destination: file.mp4: No space left on device";
        let err = EngineError::classify_from_stderr(stderr);
        match err {
            EngineError::DiskFull(_) => {
                assert!(err.user_friendly_message().contains("disk space"));
            }
            _ => panic!("Expected DiskFull variant"),
        }
    }

    #[test]
    fn test_classify_permission_denied() {
        let stderr = "ERROR: unable to open for writing: Permission denied";
        let err = EngineError::classify_from_stderr(stderr);
        match err {
            EngineError::PermissionDenied(_) => {
                assert!(err.user_friendly_message().contains("Permission denied"));
            }
            _ => panic!("Expected PermissionDenied variant"),
        }
    }
}
