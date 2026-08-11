use std::fs;
use chrono::Utc;

use siphonix_lib::db::DbRepository;
use siphonix_lib::engine::builder::DownloadRequest;
use siphonix_lib::engine::detector::EngineDetector;
use siphonix_lib::engine::DownloadManager;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::scheduler::QueueScheduler;
use siphonix_lib::queue::state::JobState;
use tokio::process::Command;

#[tokio::test]
async fn test_real_world_batch_and_crash_recovery() {
    let engine = EngineDetector::detect().expect("yt-dlp and ffmpeg must be detected");
    let test_urls = vec![
        "https://www.youtube.com/watch?v=aqz-KE-bpKQ", // Big Buck Bunny
        "https://www.youtube.com/watch?v=YE7VzlLtp-4", // Tears of Steel clip
        "https://www.youtube.com/watch?v=aqz-KE-bpKQ", // Big Buck Bunny (2nd item)
    ];

    let temp_dir = std::env::temp_dir().join(format!("siphonix_e2e_batch_{}", rand::random::<u64>()));
    let db_path = temp_dir.join("siphonix_batch.db");
    let dest_dir = temp_dir.join("downloads");
    let _ = fs::create_dir_all(&dest_dir);

    // Initialize DB and Scheduler
    let db = DbRepository::init(&db_path).await.expect("DB init failed");
    let mgr = DownloadManager::new();
    let scheduler = QueueScheduler::new(std::sync::Arc::new(db), mgr).await;

    scheduler.set_max_concurrency(2).await;

    println!("--- Step 1: Enqueueing 3 Real YouTube Download Jobs ---");
    for (idx, url) in test_urls.iter().enumerate() {
        let is_audio = idx == 1;
        let job = DownloadJob {
            id: format!("batch-job-{}", idx + 1),
            url: url.to_string(),
            title: format!("Test Batch Video {}", idx + 1),
            thumbnail_url: None,
            media_mode: if is_audio { "audio".to_string() } else { "video".to_string() },
            format: if is_audio { "MP3".to_string() } else { "MP4".to_string() },
            quality: if is_audio { "320k".to_string() } else { "720p".to_string() },
            destination_path: dest_dir.to_string_lossy().to_string(),
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
            source_video_id: Some(format!("vid-{}", idx + 1)),
            source_playlist_id: None,
            source_playlist_title: None,
            playlist_entry_index: None,
        };

        let enqueued = scheduler.enqueue_job(job).await.expect("Enqueue failed");
        assert_eq!(enqueued.state, JobState::QUEUED);
        println!("Enqueued job #{}: ID={}, Format={}", idx + 1, enqueued.id, enqueued.format);
    }

    // Verify initial queue state
    let initial_jobs = scheduler.get_all_jobs().await;
    assert_eq!(initial_jobs.len(), 3);
    assert_eq!(initial_jobs[0].state, JobState::QUEUED);
    assert_eq!(initial_jobs[1].state, JobState::QUEUED);

    println!("--- Step 2: Testing Job Cancellation Isolation ---");
    // Cancel 3rd job while queued
    let cancelled = scheduler.cancel_job("batch-job-3").await;
    assert!(cancelled, "Job 3 should be cancelled");

    let jobs_after_cancel = scheduler.get_all_jobs().await;
    assert_eq!(jobs_after_cancel[2].state, JobState::CANCELLED);
    assert_eq!(jobs_after_cancel[0].state, JobState::QUEUED, "Job 1 remains queued and unaffected");

    println!("--- Step 3: Executing Job 1 Real Download ---");
    // Execute job 1 via command builder
    let req1 = DownloadRequest {
        url: test_urls[0].to_string(),
        media_mode: "video".to_string(),
        audio_format: None,
        audio_quality: None,
        video_format: Some("MP4".to_string()),
        video_quality: Some("720p".to_string()),
        destination_path: dest_dir.to_string_lossy().to_string(),
    };

    let args1 = siphonix_lib::engine::builder::CommandBuilder::build_args(&req1, &engine);
    let mut cmd1 = Command::new(&engine.yt_dlp);
    cmd1.args(&args1);
    let out1 = cmd1.output().await.expect("Failed download 1");
    assert!(out1.status.success(), "Download 1 failed");

    scheduler.handle_progress_event("batch-job-1", "COMPLETED", 100.0, None, None, None, None).await;

    println!("--- Step 4: Testing SQLite Restart & Crash Recovery ---");
    // Simulate app crash while job 2 was DOWNLOADING
    scheduler.handle_progress_event("batch-job-2", "DOWNLOADING", 45.0, Some("2.5 MB/s"), Some("00:10"), None, None).await;

    // Drop previous repository and re-initialize from disk (simulating app restart)
    drop(scheduler);
    let db_recovered = DbRepository::init(&db_path).await.expect("DB re-init failed");
    let recovered_count = db_recovered.recover_interrupted_jobs().await.expect("Recovery query failed");

    assert_eq!(recovered_count, 1, "Interrupted DOWNLOADING job 2 must be recovered");

    let all_recovered = db_recovered.get_all_jobs().await.expect("Get all failed");
    let job1 = all_recovered.iter().find(|j| j.id == "batch-job-1").unwrap();
    let job2 = all_recovered.iter().find(|j| j.id == "batch-job-2").unwrap();
    let job3 = all_recovered.iter().find(|j| j.id == "batch-job-3").unwrap();

    assert_eq!(job1.state, "COMPLETED", "Completed job 1 remains COMPLETED in SQLite");
    assert_eq!(job2.state, "QUEUED", "Interrupted job 2 recovered to QUEUED in SQLite");
    assert_eq!(job3.state, "CANCELLED", "Cancelled job 3 remains CANCELLED in SQLite");

    println!("--- Step 5: Verifying Downloaded Media File on Disk ---");
    let files: Vec<_> = fs::read_dir(&dest_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert!(!files.is_empty(), "Downloaded media files must exist on disk");
    let file_meta = fs::metadata(files[0].path()).unwrap();
    assert!(file_meta.len() > 100_000, "Downloaded media file size must be non-zero");

    println!("Batch Test Summary:");
    println!("  Jobs Queued: 3");
    println!("  Jobs Completed: 1");
    println!("  Jobs Cancelled: 1");
    println!("  Jobs Recovered after Crash: 1");
    println!("  Downloaded File: '{}' ({} bytes)", files[0].path().display(), file_meta.len());

    let _ = fs::remove_dir_all(temp_dir);
}
