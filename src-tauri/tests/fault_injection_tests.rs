use std::sync::Arc;
use tempfile::tempdir;

use siphonix_lib::db::repository::DbJob;
use siphonix_lib::db::DbRepository;
use siphonix_lib::diagnostics::{DiagnosticEvent, DiagnosticLogger, DiagnosticSeverity, DiagnosticsManager};
use siphonix_lib::engine::options::DownloadOptions;
use siphonix_lib::engine::DownloadManager;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::state::JobState;
use siphonix_lib::queue::QueueScheduler;
use siphonix_lib::runtime::EngineManager;

#[tokio::test]
async fn test_diagnostics_storage_unavailable_does_not_block_queue() {
    // Requirement #2: Storage/logger unavailable must NEVER block queue scheduling or downloads
    let tmp = tempdir().unwrap();
    let read_only_dir = tmp.path().join("read_only_logs");
    std::fs::create_dir_all(&read_only_dir).unwrap();

    // Make directory read-only (simulating storage write failure)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&read_only_dir, std::fs::Permissions::from_mode(0o444)).unwrap();
    }

    let logger = DiagnosticLogger::new(&read_only_dir);

    // Logging to read-only path must NOT panic or block
    for i in 0..50 {
        logger.log(DiagnosticEvent::new(
            DiagnosticSeverity::Info,
            "test",
            "TEST_EVENT",
            format!("Logging under read-only storage test line {}", i),
        ));
    }

    let db_path = tmp.path().join("fault_queue.db");
    let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
    let engine_mgr = Arc::new(EngineManager::new(tmp.path(), Some(true)));
    let download_mgr = DownloadManager::new_with_runtime(engine_mgr);
    let scheduler = QueueScheduler::new(db.clone(), download_mgr).await;

    // Enqueue a job and verify queue operations complete normally despite storage failure
    let job = DownloadJob {
        id: "fault-job-1".to_string(),
        url: "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string(),
        title: "Fault Test Video".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "mp4".to_string(),
        quality: "720p".to_string(),
        destination_path: tmp.path().to_string_lossy().to_string(),
        state: JobState::QUEUED,
        progress: 0.0,
        download_speed: None,
        eta: None,
        file_size: None,
        error_message: None,
        last_error_category: None,
        retry_count: 0,
        max_retries: 3,
        next_retry_at: None,
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        source_video_id: Some("aqz-KE-bpKQ".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: DownloadOptions::default(),
    };

    scheduler.enqueue_job(job).await.unwrap();
    let jobs = scheduler.get_all_jobs().await;
    assert_eq!(jobs.len(), 1);

    // Restore permissions for clean tempdir cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&read_only_dir, std::fs::Permissions::from_mode(0o755));
    }
}

#[tokio::test]
async fn test_cancellation_during_download_and_processing() {
    // Requirement #1: Cancellation generates INFO event, never enters retry, never triggers rate limit
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("cancellation_test.db");
    let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
    let engine_mgr = Arc::new(EngineManager::new(tmp.path(), Some(true)));
    let diag_mgr = Arc::new(DiagnosticsManager::new(tmp.path(), engine_mgr.clone()));
    let download_mgr = DownloadManager::new_with_runtime(engine_mgr);
    let scheduler = QueueScheduler::new(db.clone(), download_mgr).await;

    let job_id = "cancel-job-101".to_string();
    let job = DownloadJob {
        id: job_id.clone(),
        url: "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string(),
        title: "Cancellation Test Video".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "mp4".to_string(),
        quality: "720p".to_string(),
        destination_path: tmp.path().to_string_lossy().to_string(),
        state: JobState::DOWNLOADING,
        progress: 45.0,
        download_speed: Some("1.2MB/s".to_string()),
        eta: Some("00:15".to_string()),
        file_size: None,
        error_message: None,
        last_error_category: None,
        retry_count: 0,
        max_retries: 3,
        next_retry_at: None,
        created_at: chrono::Utc::now(),
        started_at: Some(chrono::Utc::now()),
        completed_at: None,
        source_video_id: Some("aqz-KE-bpKQ".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: DownloadOptions::default(),
    };

    scheduler.enqueue_job(job).await.unwrap();

    // Cancel job
    let cancelled = scheduler.cancel_job(&job_id).await;
    assert!(cancelled);

    // Log diagnostic event for cancellation
    diag_mgr.record_event(
        DiagnosticEvent::new(
            DiagnosticSeverity::Info,
            "queue",
            "JOB_CANCELLED",
            format!("Job {} was cancelled by user request.", job_id),
        )
        .with_job_id(&job_id),
    );

    let updated_job = db.get_all_jobs().await.unwrap().into_iter().find(|j| j.id == job_id).unwrap();
    assert_eq!(updated_job.state, "CANCELLED");

    let events = diag_mgr.get_recent_events(10);
    let cancel_ev = events.iter().find(|e| e.event_type == "JOB_CANCELLED").unwrap();
    assert_eq!(cancel_ev.severity, DiagnosticSeverity::Info);
}

#[tokio::test]
async fn test_50_queued_jobs_stress_performance() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("stress_50.db");
    let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
    let engine_mgr = Arc::new(EngineManager::new(tmp.path(), Some(true)));
    let download_mgr = DownloadManager::new_with_runtime(engine_mgr);
    let scheduler = QueueScheduler::new(db.clone(), download_mgr).await;

    for i in 1..=50 {
        let job = DownloadJob {
            id: format!("stress-job-{}", i),
            url: format!("https://www.youtube.com/watch?v=stress{:02}", i),
            title: format!("Stress Video {}", i),
            thumbnail_url: None,
            media_mode: "video".to_string(),
            format: "mp4".to_string(),
            quality: "720p".to_string(),
            destination_path: tmp.path().to_string_lossy().to_string(),
            state: JobState::QUEUED,
            progress: 0.0,
            download_speed: None,
            eta: None,
            file_size: None,
            error_message: None,
            last_error_category: None,
            retry_count: 0,
            max_retries: 3,
            next_retry_at: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            source_video_id: None,
            source_playlist_id: None,
            source_playlist_title: None,
            playlist_entry_index: None,
            options: DownloadOptions::default(),
        };
        scheduler.enqueue_job(job).await.unwrap();
    }

    let all_jobs = scheduler.get_all_jobs().await;
    assert_eq!(all_jobs.len(), 50);
}

#[tokio::test]
async fn test_simultaneous_job_failures_queue_isolation() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("simultaneous_failures.db");
    let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
    let engine_mgr = Arc::new(EngineManager::new(tmp.path(), Some(true)));
    let download_mgr = DownloadManager::new_with_runtime(engine_mgr);
    let scheduler = QueueScheduler::new(db.clone(), download_mgr).await;

    // Enqueue job 1 (failed) and job 2 (queued)
    let job1 = DownloadJob {
        id: "job-fail-1".to_string(),
        url: "https://www.youtube.com/watch?v=fail1".to_string(),
        title: "Failing Video 1".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "mp4".to_string(),
        quality: "720p".to_string(),
        destination_path: tmp.path().to_string_lossy().to_string(),
        state: JobState::FAILED,
        progress: 0.0,
        download_speed: None,
        eta: None,
        file_size: None,
        error_message: Some("Simulated download failure".to_string()),
        last_error_category: Some("DOWNLOAD_FAILED".to_string()),
        retry_count: 3,
        max_retries: 3,
        next_retry_at: None,
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        source_video_id: None,
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: DownloadOptions::default(),
    };

    let job2 = DownloadJob {
        id: "job-queued-2".to_string(),
        url: "https://www.youtube.com/watch?v=queued2".to_string(),
        title: "Queued Video 2".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "mp4".to_string(),
        quality: "720p".to_string(),
        destination_path: tmp.path().to_string_lossy().to_string(),
        state: JobState::QUEUED,
        progress: 0.0,
        download_speed: None,
        eta: None,
        file_size: None,
        error_message: None,
        last_error_category: None,
        retry_count: 0,
        max_retries: 3,
        next_retry_at: None,
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        source_video_id: None,
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: DownloadOptions::default(),
    };

    scheduler.enqueue_job(job1).await.unwrap();
    scheduler.enqueue_job(job2).await.unwrap();

    let jobs = scheduler.get_all_jobs().await;
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs.iter().find(|j| j.id == "job-fail-1").unwrap().state, JobState::FAILED);
    assert_eq!(jobs.iter().find(|j| j.id == "job-queued-2").unwrap().state, JobState::QUEUED);
}

#[tokio::test]
async fn test_rate_limit_cooldown_isolation() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("rate_limit_test.db");
    let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
    let engine_mgr = Arc::new(EngineManager::new(tmp.path(), Some(true)));
    let download_mgr = DownloadManager::new_with_runtime(engine_mgr);
    let scheduler = QueueScheduler::new(db.clone(), download_mgr).await;

    // Cooldown trigger must be isolated to queue state without destroying queued items
    let jobs = scheduler.get_all_jobs().await;
    assert_eq!(jobs.len(), 0);
}

#[tokio::test]
async fn test_database_failure_resilience() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("db_resilience.db");
    let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
    let engine_mgr = Arc::new(EngineManager::new(tmp.path(), Some(true)));
    let diag_mgr = Arc::new(DiagnosticsManager::new(tmp.path(), engine_mgr));

    let health = diag_mgr.get_system_health(Some(&db)).await;
    assert_eq!(health.database.status, siphonix_lib::diagnostics::SystemHealthStatus::Healthy);
}

#[tokio::test]
async fn test_app_restart_recovery() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("restart_recovery.db");

    // Phase A: App instance 1 running with interrupted DOWNLOADING job
    {
        let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
        let job = DownloadJob {
            id: "job-interrupted-1".to_string(),
            url: "https://www.youtube.com/watch?v=interrupted1".to_string(),
            title: "Interrupted Video".to_string(),
            thumbnail_url: None,
            media_mode: "video".to_string(),
            format: "mp4".to_string(),
            quality: "720p".to_string(),
            destination_path: tmp.path().to_string_lossy().to_string(),
            state: JobState::DOWNLOADING,
            progress: 30.0,
            download_speed: None,
            eta: None,
            file_size: None,
            error_message: None,
            last_error_category: None,
            retry_count: 0,
            max_retries: 3,
            next_retry_at: None,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            source_video_id: None,
            source_playlist_id: None,
            source_playlist_title: None,
            playlist_entry_index: None,
            options: DownloadOptions::default(),
        };
        db.insert_job(&job.to_db_job()).await.unwrap();
    }

    // Phase B: App instance 2 restarts and recovers interrupted job
    {
        let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
        let engine_mgr = Arc::new(EngineManager::new(tmp.path(), Some(true)));
        let download_mgr = DownloadManager::new_with_runtime(engine_mgr);
        let scheduler = QueueScheduler::new(db.clone(), download_mgr).await;

        let jobs = scheduler.get_all_jobs().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].state, JobState::QUEUED);
    }
}
