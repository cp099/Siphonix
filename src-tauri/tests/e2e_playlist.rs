use std::fs;
use chrono::Utc;

use siphonix_lib::commands::url::{validate_url, UrlType};
use siphonix_lib::db::DbRepository;
use siphonix_lib::engine::detector::EngineDetector;
use siphonix_lib::engine::playlist::PlaylistInspector;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::state::JobState;

#[tokio::test]
async fn test_url_classification_video_vs_playlist() {
    let single_vid = validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());
    assert_eq!(single_vid.url_type, UrlType::VIDEO);
    assert!(!single_vid.is_playlist);

    let pure_pl = validate_url("https://www.youtube.com/playlist?list=PL3rVcngGfeeqE5H9N9".to_string());
    assert_eq!(pure_pl.url_type, UrlType::PLAYLIST);
    assert!(pure_pl.is_playlist);

    let vid_with_pl = validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PL3rVcngGfeeqE5H9N9".to_string());
    assert_eq!(vid_with_pl.url_type, UrlType::VIDEO_WITH_PLAYLIST);
    assert!(vid_with_pl.is_playlist);
    assert_eq!(vid_with_pl.video_id, Some("dQw4w9WgXcQ".to_string()));
    assert_eq!(vid_with_pl.playlist_id, Some("PL3rVcngGfeeqE5H9N9".to_string()));
}

#[tokio::test]
async fn test_real_playlist_inspection() {
    let engine = EngineDetector::detect().expect("yt-dlp must be present");
    let inspector = PlaylistInspector::new();

    let test_pl_url = "https://www.youtube.com/@BlenderOfficial/videos";
    let info = inspector.inspect_playlist("insp-test-1", test_pl_url, &engine)
        .await
        .expect("Playlist inspection failed");

    assert!(!info.id.is_empty(), "Playlist ID must be populated");
    assert!(!info.entries.is_empty(), "Playlist entries should be returned");
    assert_eq!(info.entries[0].index, 1, "First entry must preserve 1-based index");

    println!("Real Playlist Inspection OK: Title='{}', Entries={}", info.title, info.entry_count);
}

#[tokio::test]
async fn test_playlist_duplicate_detection() {
    let temp_dir = std::env::temp_dir().join(format!("siphonix_pl_db_{}", rand::random::<u64>()));
    let db_path = temp_dir.join("test.db");
    let db = DbRepository::init(&db_path).await.expect("DB init failed");

    // Insert an existing completed job
    let existing_job = DownloadJob {
        id: "dup-job-1".to_string(),
        url: "https://www.youtube.com/watch?v=v-dup-123".to_string(),
        title: "Existing Download".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "1080p".to_string(),
        destination_path: "/tmp".to_string(),
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
        source_video_id: Some("v-dup-123".to_string()),
        source_playlist_id: Some("PL3rVcngGfeeqE5H9N9".to_string()),
        source_playlist_title: Some("Test Playlist".to_string()),
        playlist_entry_index: Some(1),
        options: Default::default(),
    };

    db.insert_job(&existing_job.to_db_job()).await.unwrap();

    let check_ids = vec!["v-dup-123".to_string(), "v-new-456".to_string()];
    let existing_set = db.find_existing_video_ids(&check_ids).await.unwrap();

    assert!(existing_set.contains("v-dup-123"), "v-dup-123 must be flagged as existing");
    assert!(!existing_set.contains("v-new-456"), "v-new-456 is new and not flagged");

    let _ = fs::remove_dir_all(temp_dir);
}
