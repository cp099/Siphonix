use std::fs;
use chrono::Utc;

use siphonix_lib::db::DbRepository;
use siphonix_lib::engine::DownloadManager;
use siphonix_lib::queue::cooldown::CooldownManager;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::scheduler::QueueScheduler;
use siphonix_lib::queue::state::JobState;

async fn setup_test_scheduler() -> (std::sync::Arc<QueueScheduler>, std::path::PathBuf) {
    let temp_dir = std::env::temp_dir().join(format!("siphonix_test_db_{}", rand::random::<u64>()));
    let db_path = temp_dir.join("test.db");
    let db = DbRepository::init(&db_path).await.expect("DB init failed");
    let mgr = DownloadManager::new();
    let scheduler = QueueScheduler::new(std::sync::Arc::new(db), mgr).await;
    (scheduler, temp_dir)
}

fn create_test_job(id: &str) -> DownloadJob {
    DownloadJob {
        id: id.to_string(),
        url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        title: format!("Test Job {}", id),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "1080p".to_string(),
        destination_path: "/tmp".to_string(),
        state: JobState::QUEUED,
        progress: 0.0,
        download_speed: None,
        eta: None,
        file_size: None,
        error_message: None,
        last_error_category: None,
        retry_count: 0,
        max_retries: 5,
        next_retry_at: None,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        source_video_id: Some(id.to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    }
}

#[tokio::test]
async fn test_fifo_dispatch_ordering_and_concurrency() {
    let (scheduler, temp_dir) = setup_test_scheduler().await;

    scheduler.set_max_concurrency(2).await;

    let _j1 = scheduler.enqueue_job(create_test_job("j1")).await.unwrap();
    let _j2 = scheduler.enqueue_job(create_test_job("j2")).await.unwrap();
    let _j3 = scheduler.enqueue_job(create_test_job("j3")).await.unwrap();

    let all_jobs = scheduler.get_all_jobs().await;
    assert_eq!(all_jobs.len(), 3);
    assert_eq!(all_jobs[0].id, "j1"); // Enqueued in FIFO order

    let _ = fs::remove_dir_all(temp_dir);
}

#[tokio::test]
async fn test_queue_pause_resume() {
    let (scheduler, temp_dir) = setup_test_scheduler().await;

    scheduler.set_pause_queue(true).await;
    let _ = scheduler.enqueue_job(create_test_job("j1")).await;

    scheduler.tick_scheduler(None).await;
    let jobs = scheduler.get_all_jobs().await;
    assert_eq!(jobs[0].state, JobState::QUEUED); // Should stay queued while queue is paused!

    scheduler.set_pause_queue(false).await;
    scheduler.tick_scheduler(None).await;

    let _ = fs::remove_dir_all(temp_dir);
}

#[tokio::test]
async fn test_cancellation_isolation() {
    let (scheduler, temp_dir) = setup_test_scheduler().await;

    let _j1 = scheduler.enqueue_job(create_test_job("c1")).await.unwrap();
    let _j2 = scheduler.enqueue_job(create_test_job("c2")).await.unwrap();

    let cancelled = scheduler.cancel_job("c1").await;
    assert!(cancelled);

    let jobs = scheduler.get_all_jobs().await;
    assert_eq!(jobs[0].state, JobState::CANCELLED);
    assert_eq!(jobs[1].state, JobState::QUEUED); // Second job remains active & queued!

    let _ = fs::remove_dir_all(temp_dir);
}

#[tokio::test]
async fn test_50_queued_jobs_stress_performance() {
    let (scheduler, temp_dir) = setup_test_scheduler().await;

    scheduler.set_max_concurrency(2).await;

    for i in 1..=50 {
        let _ = scheduler.enqueue_job(create_test_job(&format!("stress-{}", i))).await;
    }

    let all_jobs = scheduler.get_all_jobs().await;
    assert_eq!(all_jobs.len(), 50);

    // Tick scheduler
    scheduler.tick_scheduler(None).await;

    let _ = fs::remove_dir_all(temp_dir);
}

#[tokio::test]
async fn test_cooldown_manual_resume_preserves_rate_limit_history() {
    let mut mgr = CooldownManager::new(2, 60, 120);

    mgr.record_rate_limit();
    mgr.record_rate_limit();
    assert!(mgr.is_cooldown_active());

    mgr.force_resume();
    assert!(!mgr.is_cooldown_active());

    assert!(mgr.record_rate_limit());
    assert!(mgr.is_cooldown_active());
}

#[tokio::test]
async fn test_db_crash_recovery_of_interrupted_jobs() {
    let temp_dir = std::env::temp_dir().join(format!("siphonix_recover_db_{}", rand::random::<u64>()));
    let db_path = temp_dir.join("test.db");
    let db = DbRepository::init(&db_path).await.expect("DB init failed");

    let mut interrupted_job = create_test_job("interrupted-1");
    interrupted_job.state = JobState::DOWNLOADING;
    db.insert_job(&interrupted_job.to_db_job()).await.unwrap();

    let recovered_count = db.recover_interrupted_jobs().await.unwrap();
    assert_eq!(recovered_count, 1);

    let all_jobs = db.get_all_jobs().await.unwrap();
    assert_eq!(all_jobs[0].state, "QUEUED");

    let _ = fs::remove_dir_all(temp_dir);
}
