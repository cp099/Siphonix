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

        if stderr_lower.contains("video unavailable") || stderr_lower.contains("private video") || stderr_lower.contains("has been removed") {
            Self::VideoUnavailable(stderr.to_string())
        } else if stderr_lower.contains("http error 429") || stderr_lower.contains("too many requests") {
            Self::RateLimited(stderr.to_string())
        } else if stderr_lower.contains("sign in to confirm your age") || stderr_lower.contains("members-only") {
            Self::AuthenticationRequired(stderr.to_string())
        } else if stderr_lower.contains("permission denied") || stderr_lower.contains("access is denied") {
            Self::PermissionDenied(stderr.to_string())
        } else if stderr_lower.contains("no space left on device") {
            Self::DiskFull(stderr.to_string())
        } else if stderr_lower.contains("requested format is not available") {
            Self::FormatUnavailable(stderr.to_string())
        } else if stderr_lower.contains("ffmpeg") && (stderr_lower.contains("error") || stderr_lower.contains("failed")) {
            Self::FFmpegProcessingError(stderr.to_string())
        } else if stderr_lower.contains("unable to download webpage") || stderr_lower.contains("connection timed out") || stderr_lower.contains("network is unreachable") {
            Self::TemporaryNetworkError(stderr.to_string())
        } else {
            Self::ProcessFailed {
                code: None,
                stderr: stderr.to_string(),
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
}

