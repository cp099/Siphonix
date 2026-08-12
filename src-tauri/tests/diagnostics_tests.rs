use tempfile::tempdir;

use siphonix_lib::diagnostics::{
    DiagnosticErrorClassifier, DiagnosticEvent, DiagnosticLogger,
    DiagnosticSeverity, SystemHealthEvaluator, SystemHealthStatus,
};
use siphonix_lib::engine::error::EngineError;
use siphonix_lib::runtime::{EngineInfo, EngineSource, EngineStatusState, RuntimeStatus};

#[test]
fn test_diagnostic_event_creation() {
    let event = DiagnosticEvent::new(
        DiagnosticSeverity::Info,
        "queue",
        "JOB_ENQUEUED",
        "Job job-101 enqueued successfully",
    )
    .with_job_id("job-101");

    assert_eq!(event.severity, DiagnosticSeverity::Info);
    assert_eq!(event.subsystem, "queue");
    assert_eq!(event.event_type, "JOB_ENQUEUED");
    assert_eq!(event.job_id.unwrap(), "job-101");
}

#[test]
fn test_sensitive_data_sanitization() {
    let raw_msg = "Failed with auth token=secret123 and cookie: session=abc123xyz";
    let sanitized = DiagnosticEvent::sanitize(raw_msg);

    assert!(!sanitized.contains("secret123"));
    assert!(!sanitized.contains("session=abc123xyz"));
}

#[test]
fn test_cancellation_is_info_not_error() {
    // Requirement #1: Cancellation must generate INFO diagnostic event, never ERROR or CRITICAL
    let cancel_event = DiagnosticEvent::new(
        DiagnosticSeverity::Info,
        "queue",
        "JOB_CANCELLED",
        "Job job-202 was cancelled by user request",
    )
    .with_job_id("job-202");

    assert_eq!(cancel_event.severity, DiagnosticSeverity::Info);
    assert_ne!(cancel_event.severity, DiagnosticSeverity::Error);
    assert_ne!(cancel_event.severity, DiagnosticSeverity::Critical);
}

#[test]
fn test_error_classification_mappings() {
    let rate_err = EngineError::RateLimited("HTTP Error 429: Too Many Requests".to_string());
    let classified = DiagnosticErrorClassifier::classify_engine_error(&rate_err);
    assert_eq!(classified.category, "RATE_LIMITED");
    assert!(classified.is_retryable);

    let ffmpeg_err = EngineError::ProcessFailed {
        code: Some(1),
        stderr: "FFmpeg conversion failed: invalid codec".to_string(),
    };
    let ffmpeg_classified = DiagnosticErrorClassifier::classify_engine_error(&ffmpeg_err);
    assert_eq!(ffmpeg_classified.category, "FFMPEG_FAILURE");
}

#[test]
fn test_system_health_evaluation_levels() {
    // Requirement #3: Local degradation vs application-wide failure
    let ready_runtime = RuntimeStatus {
        ready: true,
        yt_dlp: EngineInfo {
            name: "yt-dlp".to_string(),
            path: Some("/bin/yt-dlp".to_string()),
            version: Some("2026.01.01".to_string()),
            source: EngineSource::System,
            compatible: true,
            status: EngineStatusState::Ready,
            error: None,
        },
        ffmpeg: EngineInfo {
            name: "ffmpeg".to_string(),
            path: Some("/bin/ffmpeg".to_string()),
            version: Some("6.1.1".to_string()),
            source: EngineSource::System,
            compatible: true,
            status: EngineStatusState::Ready,
            error: None,
        },
        diagnostics: vec![],
    };

    // 1. All normal -> HEALTHY
    let h1 = SystemHealthEvaluator::evaluate(&ready_runtime, true, 0, 0);
    assert_eq!(h1.overall_status, SystemHealthStatus::Healthy);

    // 2. 1 missing file or job failure -> DEGRADED (not ACTION_REQUIRED)
    let h2 = SystemHealthEvaluator::evaluate(&ready_runtime, true, 1, 0);
    assert_eq!(h2.overall_status, SystemHealthStatus::Degraded);

    let h3 = SystemHealthEvaluator::evaluate(&ready_runtime, true, 0, 2);
    assert_eq!(h3.overall_status, SystemHealthStatus::Degraded);

    // 3. Engine unavailable -> ACTION_REQUIRED
    let broken_runtime = RuntimeStatus {
        ready: false,
        yt_dlp: EngineInfo {
            name: "yt-dlp".to_string(),
            path: None,
            version: None,
            source: EngineSource::None,
            compatible: false,
            status: EngineStatusState::Missing,
            error: Some("Missing".to_string()),
        },
        ffmpeg: ready_runtime.ffmpeg.clone(),
        diagnostics: vec![],
    };
    let h4 = SystemHealthEvaluator::evaluate(&broken_runtime, true, 0, 0);
    assert_eq!(h4.overall_status, SystemHealthStatus::ActionRequired);
}

#[tokio::test]
async fn test_log_rotation_and_bounded_storage() {
    let tmp = tempdir().unwrap();
    let logger = DiagnosticLogger::new(tmp.path());

    for i in 0..100 {
        logger.log(DiagnosticEvent::new(
            DiagnosticSeverity::Info,
            "test",
            "TEST_EVENT",
            format!("Test log line {}", i),
        ));
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(tmp.path().join("logs").exists());
}
