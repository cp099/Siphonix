use std::fs::File;
use std::sync::Arc;
use tempfile::tempdir;
use siphonix_lib::db::DbRepository;
use siphonix_lib::library::LibraryService;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::state::JobState;
use chrono::Utc;

#[tokio::test]
async fn test_insert_and_query_library_items() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_lib.db");
    let db = DbRepository::init(&db_path).await.unwrap();
    let db_arc = Arc::new(db);
    let service = LibraryService::new(db_arc.clone());

    // Create physical test file
    let media_path = dir.path().join("Test Video.mp4");
    File::create(&media_path).unwrap();

    let job = DownloadJob {
        id: "job-101".to_string(),
        url: "https://www.youtube.com/watch?v=video101".to_string(),
        title: "Test Video".to_string(),
        thumbnail_url: Some("https://img.youtube.com/vi/video101/0.jpg".to_string()),
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "1080p".to_string(),
        destination_path: dir.path().to_string_lossy().to_string(),
        state: JobState::COMPLETED,
        progress: 100.0,
        download_speed: None,
        eta: None,
        file_size: Some("10 MB".to_string()),
        error_message: None,
        last_error_category: None,
        retry_count: 0,
        max_retries: 5,
        next_retry_at: None,
        created_at: Utc::now(),
        started_at: Some(Utc::now()),
        completed_at: Some(Utc::now()),
        source_video_id: Some("video101".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    // Insert job into jobs table first
    db_arc.insert_job(&job.to_db_job()).await.unwrap();

    let item = service.register_completed_job(&job, Some(&media_path.to_string_lossy())).await.unwrap();
    assert_eq!(item.title, "Test Video");
    assert_eq!(item.file_status, "AVAILABLE");

    // Query library items
    let items = service.get_library_items(Some("Test".to_string()), Some("video".to_string()), Some("AVAILABLE".to_string()), Some("newest".to_string())).await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].source_video_id, Some("video101".to_string()));
}

#[tokio::test]
async fn test_identical_titles_different_video_ids() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_lib_duplicate_titles.db");
    let db = DbRepository::init(&db_path).await.unwrap();
    let db_arc = Arc::new(db);
    let service = LibraryService::new(db_arc.clone());

    let file_1 = dir.path().join("Duplicate Title [videoA].mp4");
    let file_2 = dir.path().join("Duplicate Title [videoB].mp4");
    File::create(&file_1).unwrap();
    File::create(&file_2).unwrap();

    let job1 = DownloadJob {
        id: "job-A".to_string(),
        url: "https://www.youtube.com/watch?v=videoA".to_string(),
        title: "Duplicate Title".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "1080p".to_string(),
        destination_path: dir.path().to_string_lossy().to_string(),
        state: JobState::COMPLETED,
        progress: 100.0,
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
        completed_at: Some(Utc::now()),
        source_video_id: Some("videoA".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    let mut job2 = job1.clone();
    job2.id = "job-B".to_string();
    job2.url = "https://www.youtube.com/watch?v=videoB".to_string();
    job2.source_video_id = Some("videoB".to_string());

    db_arc.insert_job(&job1.to_db_job()).await.unwrap();
    db_arc.insert_job(&job2.to_db_job()).await.unwrap();

    let item1 = service.register_completed_job(&job1, Some(&file_1.to_string_lossy())).await.unwrap();
    let item2 = service.register_completed_job(&job2, Some(&file_2.to_string_lossy())).await.unwrap();

    assert_ne!(item1.id, item2.id);
    assert_ne!(item1.file_path, item2.file_path);

    let all_items = service.get_library_items(None, None, None, None).await.unwrap();
    assert_eq!(all_items.len(), 2);
}

#[tokio::test]
async fn test_missing_file_detection_after_external_deletion() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_lib_missing.db");
    let db = DbRepository::init(&db_path).await.unwrap();
    let db_arc = Arc::new(db);
    let service = LibraryService::new(db_arc.clone());

    let file_path = dir.path().join("ToBeDeleted.mp3");
    File::create(&file_path).unwrap();

    let job = DownloadJob {
        id: "job-del".to_string(),
        url: "https://www.youtube.com/watch?v=vdel".to_string(),
        title: "ToBeDeleted".to_string(),
        thumbnail_url: None,
        media_mode: "audio".to_string(),
        format: "MP3".to_string(),
        quality: "320k".to_string(),
        destination_path: dir.path().to_string_lossy().to_string(),
        state: JobState::COMPLETED,
        progress: 100.0,
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
        completed_at: Some(Utc::now()),
        source_video_id: Some("vdel".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    db_arc.insert_job(&job.to_db_job()).await.unwrap();
    let item = service.register_completed_job(&job, Some(&file_path.to_string_lossy())).await.unwrap();
    assert_eq!(item.file_status, "AVAILABLE");

    // Externally delete physical file on disk
    std::fs::remove_file(&file_path).unwrap();

    // Verify status update
    let updated_items = service.verify_library_items().await.unwrap();
    assert_eq!(updated_items.len(), 1);
    assert_eq!(updated_items[0].file_status, "MISSING");
}

#[tokio::test]
async fn test_exact_path_deletion_protection() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_lib_security.db");
    let db = DbRepository::init(&db_path).await.unwrap();
    let db_arc = Arc::new(db);
    let service = LibraryService::new(db_arc.clone());

    let file_path = dir.path().join("SafeFile.mp4");
    File::create(&file_path).unwrap();

    let job = DownloadJob {
        id: "job-sec".to_string(),
        url: "https://www.youtube.com/watch?v=vsec".to_string(),
        title: "SafeFile".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "1080p".to_string(),
        destination_path: dir.path().to_string_lossy().to_string(),
        state: JobState::COMPLETED,
        progress: 100.0,
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
        completed_at: Some(Utc::now()),
        source_video_id: Some("vsec".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    db_arc.insert_job(&job.to_db_job()).await.unwrap();
    let item = service.register_completed_job(&job, Some(&file_path.to_string_lossy())).await.unwrap();

    // Delete item file through service
    service.delete_item_file(&item.id).await.unwrap();

    assert!(!file_path.exists());
    let items = service.get_library_items(None, None, None, None).await.unwrap();
    assert_eq!(items.len(), 0);

    // Verify original DownloadJob remains in jobs table
    let jobs = db_arc.get_all_jobs().await.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "job-sec");
}

#[tokio::test]
async fn test_remove_record_preserves_file_and_job_history() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_lib_remove.db");
    let db = DbRepository::init(&db_path).await.unwrap();
    let db_arc = Arc::new(db);
    let service = LibraryService::new(db_arc.clone());

    let file_path = dir.path().join("PreservedFile.mp4");
    File::create(&file_path).unwrap();

    let job = DownloadJob {
        id: "job-pres".to_string(),
        url: "https://www.youtube.com/watch?v=vpres".to_string(),
        title: "PreservedFile".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "1080p".to_string(),
        destination_path: dir.path().to_string_lossy().to_string(),
        state: JobState::COMPLETED,
        progress: 100.0,
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
        completed_at: Some(Utc::now()),
        source_video_id: Some("vpres".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    db_arc.insert_job(&job.to_db_job()).await.unwrap();
    let item = service.register_completed_job(&job, Some(&file_path.to_string_lossy())).await.unwrap();

    // Remove record only
    service.remove_item_record(&item.id).await.unwrap();

    // Physical file must still exist
    assert!(file_path.exists());

    // Library items count is 0
    let items = service.get_library_items(None, None, None, None).await.unwrap();
    assert_eq!(items.len(), 0);

    // Download history job remains intact
    let jobs = db_arc.get_all_jobs().await.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "job-pres");
}
