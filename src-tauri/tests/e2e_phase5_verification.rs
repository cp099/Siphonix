use std::fs::File;
use std::sync::Arc;
use tempfile::tempdir;
use chrono::Utc;

use siphonix_lib::db::DbRepository;
use siphonix_lib::engine::builder::DownloadRequest;
use siphonix_lib::engine::detector::EngineDetector;
use siphonix_lib::engine::DownloadManager;
use siphonix_lib::engine::playlist::PlaylistInspector;
use siphonix_lib::library::LibraryService;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::scheduler::QueueScheduler;
use siphonix_lib::queue::state::JobState;

#[tokio::test]
async fn test_real_world_phase5_library_verification() {
    let dev_mode = std::env::var("SIPHONIX_DEV_INSECURE_SSL").is_ok();

    let dir = tempdir().expect("Failed to create tempdir");
    let db_path = dir.path().join("phase5_verification.db");

    println!("=== Phase 5 Real-World Verification Test Suite ===");
    println!("Database Path: {:?}", db_path);

    // Init DB & Download Engine
    let db = DbRepository::init(&db_path).await.expect("Failed DB init");
    let db_arc = Arc::new(db);
    let manager = DownloadManager::new();
    let scheduler = QueueScheduler::new(db_arc.clone(), manager.clone()).await;
    let library_service = LibraryService::new(db_arc.clone());

    // ----------------------------------------------------
    // TEST 1: Real Video -> Library with after_move:filepath
    // ----------------------------------------------------
    println!("\n--- Test 1: Real Video Download & Exact Path Library Registration ---");
    let test_video_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";
    let video_dest = dir.path().join("video_downloads");
    std::fs::create_dir_all(&video_dest).unwrap();

    let video_job_id = "phase5-video-1".to_string();
    let video_job = DownloadJob {
        id: video_job_id.clone(),
        url: test_video_url.to_string(),
        title: "Big Buck Bunny 60fps 4K".to_string(),
        thumbnail_url: Some("https://i.ytimg.com/vi/aqz-KE-bpKQ/hqdefault.jpg".to_string()),
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "1080p".to_string(),
        destination_path: video_dest.to_string_lossy().to_string(),
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
        source_video_id: Some("aqz-KE-bpKQ".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    db_arc.insert_job(&video_job.to_db_job()).await.unwrap();

    // Execute real yt-dlp video download
    let engine = manager.ensure_engine().await.expect("Engine detection failed");
    let req_video = DownloadRequest {
        url: test_video_url.to_string(),
        media_mode: "video".to_string(),
        audio_format: None,
        audio_quality: None,
        video_format: Some("MP4".to_string()),
        video_quality: Some("1080p".to_string()),
        destination_path: video_dest.to_string_lossy().to_string(),
        options: None,
    };

    let mut args = siphonix_lib::engine::builder::CommandBuilder::build_args(&req_video, &engine);
    if dev_mode {
        args.push("--no-check-certificates".to_string());
    }

    let mut cmd = tokio::process::Command::new(&engine.yt_dlp);
    cmd.args(&args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn yt-dlp video download");
    let stdout = child.stdout.take().unwrap();

    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut reader = BufReader::new(stdout).lines();
    let mut captured_video_path: Option<String> = None;

    while let Ok(Some(line)) = reader.next_line().await {
        if siphonix_lib::engine::runner::OutputParser::is_filepath_line(&line) {
            captured_video_path = Some(line.trim().to_string());
        }
    }

    let status = child.wait().await.expect("Wait failed");
    assert!(status.success(), "Real video download failed");
    assert!(captured_video_path.is_some(), "--print after_move:filepath did not capture final video output path");

    let final_video_path = captured_video_path.unwrap();
    println!("Captured Final Video Path: {}", final_video_path);
    assert!(std::path::Path::new(&final_video_path).exists(), "Captured video file does not exist on disk");

    // Register video in Library
    let registered_video = library_service
        .register_completed_job(&video_job, Some(&final_video_path))
        .await
        .expect("Failed to register video in library");

    assert_eq!(registered_video.file_path, final_video_path);
    assert_eq!(registered_video.file_status, "AVAILABLE");
    assert!(registered_video.file_size_bytes > 0, "File size bytes must be > 0");

    // Test Open & Reveal IPC logic
    library_service.open_item(&registered_video.id).await.expect("open_item failed");
    library_service.reveal_item(&registered_video.id).await.expect("reveal_item failed");
    println!("Verified Video Download -> Library Registration & Open/Reveal Actions");

    // ----------------------------------------------------
    // TEST 2: Real Audio -> Library with FFmpeg Extraction
    // ----------------------------------------------------
    println!("\n--- Test 2: Real Audio Download & MP3 Extraction Library Registration ---");
    let audio_dest = dir.path().join("audio_downloads");
    std::fs::create_dir_all(&audio_dest).unwrap();

    let audio_job_id = "phase5-audio-1".to_string();
    let audio_job = DownloadJob {
        id: audio_job_id.clone(),
        url: test_video_url.to_string(),
        title: "Big Buck Bunny Audio".to_string(),
        thumbnail_url: Some("https://i.ytimg.com/vi/aqz-KE-bpKQ/hqdefault.jpg".to_string()),
        media_mode: "audio".to_string(),
        format: "MP3".to_string(),
        quality: "320k".to_string(),
        destination_path: audio_dest.to_string_lossy().to_string(),
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
        source_video_id: Some("aqz-KE-bpKQ".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    db_arc.insert_job(&audio_job.to_db_job()).await.unwrap();

    let req_audio = DownloadRequest {
        url: test_video_url.to_string(),
        media_mode: "audio".to_string(),
        audio_format: Some("MP3".to_string()),
        audio_quality: Some("320k".to_string()),
        video_format: None,
        video_quality: None,
        destination_path: audio_dest.to_string_lossy().to_string(),
        options: None,
    };

    let mut args_audio = siphonix_lib::engine::builder::CommandBuilder::build_args(&req_audio, &engine);
    if dev_mode {
        args_audio.push("--no-check-certificates".to_string());
    }

    let mut cmd_audio = tokio::process::Command::new(&engine.yt_dlp);
    cmd_audio.args(&args_audio);
    cmd_audio.stdout(std::process::Stdio::piped());
    cmd_audio.stderr(std::process::Stdio::piped());

    let mut child_audio = cmd_audio.spawn().expect("Failed to spawn audio download");
    let stdout_audio = child_audio.stdout.take().unwrap();

    let mut reader_audio = BufReader::new(stdout_audio).lines();
    let mut captured_audio_path: Option<String> = None;

    while let Ok(Some(line)) = reader_audio.next_line().await {
        if siphonix_lib::engine::runner::OutputParser::is_filepath_line(&line) {
            captured_audio_path = Some(line.trim().to_string());
        }
    }

    let status_audio = child_audio.wait().await.expect("Wait audio failed");
    assert!(status_audio.success(), "Real audio download failed");
    assert!(captured_audio_path.is_some(), "--print after_move:filepath did not capture final MP3 path");

    let final_audio_path = captured_audio_path.unwrap();
    println!("Captured Final Audio Path: {}", final_audio_path);
    assert!(final_audio_path.ends_with(".mp3"), "Final output file path must end with .mp3");
    assert!(std::path::Path::new(&final_audio_path).exists(), "Extracted MP3 file does not exist on disk");

    let registered_audio = library_service
        .register_completed_job(&audio_job, Some(&final_audio_path))
        .await
        .expect("Failed to register audio in library");

    assert_eq!(registered_audio.file_extension, "mp3");
    assert_eq!(registered_audio.format, "MP3");
    println!("Verified Audio FFmpeg Conversion -> MP3 Library Registration");

    // ----------------------------------------------------
    // TEST 3: Real Playlist Provenance Integration
    // ----------------------------------------------------
    println!("\n--- Test 3: Real Playlist Provenance Integration ---");
    let playlist_url = "https://www.youtube.com/@BlenderOfficial/videos";
    let inspector = PlaylistInspector::new();
    let playlist_info = inspector
        .inspect_playlist("pl-inspect-1", playlist_url, &engine)
        .await
        .expect("Playlist inspection failed");

    println!("Playlist Inspected: Title='{}', Entries={}", playlist_info.title, playlist_info.entries.len());

    let entry = &playlist_info.entries[0];
    let playlist_job = DownloadJob {
        id: "phase5-playlist-job-1".to_string(),
        url: format!("https://www.youtube.com/watch?v={}", entry.id),
        title: entry.title.clone(),
        thumbnail_url: entry.thumbnail_url.clone(),
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "720p".to_string(),
        destination_path: video_dest.to_string_lossy().to_string(),
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
        started_at: Some(Utc::now()),
        completed_at: Some(Utc::now()),
        source_video_id: Some(entry.id.clone()),
        source_playlist_id: Some(playlist_info.id.clone()),
        source_playlist_title: Some(playlist_info.title.clone()),
        playlist_entry_index: Some(entry.index as u32),
        options: Default::default(),
    };

    db_arc.insert_job(&playlist_job.to_db_job()).await.unwrap();

    // Use dummy physical file for playlist item to verify provenance fields
    let pl_file_path = video_dest.join(format!("{}.mp4", entry.id));
    File::create(&pl_file_path).unwrap();

    let registered_pl_item = library_service
        .register_completed_job(&playlist_job, Some(&pl_file_path.to_string_lossy()))
        .await
        .expect("Failed to register playlist item");

    assert_eq!(registered_pl_item.source_playlist_id, Some(playlist_info.id.clone()));
    assert_eq!(registered_pl_item.source_playlist_title, Some(playlist_info.title.clone()));
    assert_eq!(registered_pl_item.playlist_entry_index, Some(entry.index as i64));
    println!("Verified Playlist Provenance: Title='{}', Index=#{}", playlist_info.title, entry.index);

    // ----------------------------------------------------
    // TEST 4: Real Missing File Detection
    // ----------------------------------------------------
    println!("\n--- Test 4: Real Missing File Detection ---");
    let file_to_delete = dir.path().join("external_deleted.mp4");
    File::create(&file_to_delete).unwrap();

    let missing_job = DownloadJob {
        id: "phase5-missing-1".to_string(),
        url: "https://www.youtube.com/watch?v=missing123".to_string(),
        title: "Externally Deleted Video".to_string(),
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
        source_video_id: Some("missing123".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: Default::default(),
    };

    db_arc.insert_job(&missing_job.to_db_job()).await.unwrap();
    let reg_missing = library_service
        .register_completed_job(&missing_job, Some(&file_to_delete.to_string_lossy()))
        .await
        .unwrap();

    assert_eq!(reg_missing.file_status, "AVAILABLE");

    // Externally delete physical file on disk using std::fs::remove_file
    std::fs::remove_file(&file_to_delete).expect("Failed external file delete");

    // Re-verify status
    let verified_items = library_service.verify_library_items().await.unwrap();
    let missing_item = verified_items.iter().find(|i| i.id == reg_missing.id).unwrap();

    assert_eq!(missing_item.file_status, "MISSING");

    // Verify DownloadJob history in jobs table is NOT deleted
    let historical_job = db_arc.get_all_jobs().await.unwrap().into_iter().find(|j| j.id == "phase5-missing-1").unwrap();
    assert_eq!(historical_job.state, "COMPLETED");
    println!("Verified Missing File Detection & DownloadJob History Preservation");

    // ----------------------------------------------------
    // TEST 5: Real Remove from Library vs Delete File
    // ----------------------------------------------------
    println!("\n--- Test 5: Real Remove from Library vs Delete File ---");
    let path_a = dir.path().join("File_A_Remove.mp4");
    let path_b = dir.path().join("File_B_Delete.mp4");
    File::create(&path_a).unwrap();
    File::create(&path_b).unwrap();

    let mut job_a = missing_job.clone();
    job_a.id = "job-file-a".to_string();
    job_a.title = "File A".to_string();

    let mut job_b = missing_job.clone();
    job_b.id = "job-file-b".to_string();
    job_b.title = "File B".to_string();

    db_arc.insert_job(&job_a.to_db_job()).await.unwrap();
    db_arc.insert_job(&job_b.to_db_job()).await.unwrap();

    let item_a = library_service.register_completed_job(&job_a, Some(&path_a.to_string_lossy())).await.unwrap();
    let item_b = library_service.register_completed_job(&job_b, Some(&path_b.to_string_lossy())).await.unwrap();

    // Action A: Remove from Library (preserves disk file)
    library_service.remove_item_record(&item_a.id).await.expect("remove_item_record failed");
    assert!(path_a.exists(), "File A on disk must still exist after Remove from Library");
    assert!(db_arc.get_library_item_by_id(&item_a.id).await.unwrap().is_none(), "File A record must be removed from DB");

    // Action B: Delete File (permanently deletes disk file and removes DB record)
    library_service.delete_item_file(&item_b.id).await.expect("delete_item_file failed");
    assert!(!path_b.exists(), "File B on disk must be deleted after Delete File");
    assert!(db_arc.get_library_item_by_id(&item_b.id).await.unwrap().is_none(), "File B record must be removed from DB");
    println!("Verified Remove from Library (file preserved) vs Delete File (file deleted)");

    // ----------------------------------------------------
    // TEST 6: Real SQLite Restart Persistence
    // ----------------------------------------------------
    println!("\n--- Test 6: Real SQLite Application Restart Persistence ---");
    // Drop existing service and pool references
    drop(library_service);
    drop(scheduler);
    drop(db_arc);

    // Re-initialize DbRepository from disk file (simulating complete app restart)
    let restarted_db = DbRepository::init(&db_path).await.expect("Failed restarted DB init");
    let restarted_db_arc = Arc::new(restarted_db);
    let restarted_service = LibraryService::new(restarted_db_arc.clone());

    let persisted_items = restarted_service.get_library_items(None, None, None, None).await.unwrap();
    assert!(!persisted_items.is_empty(), "Library items must persist across restarts");

    let restarted_video_item = persisted_items.iter().find(|i| i.id == registered_video.id).expect("Video item missing after restart");
    assert_eq!(restarted_video_item.file_path, final_video_path);
    assert_eq!(restarted_video_item.file_status, "AVAILABLE");

    let restarted_audio_item = persisted_items.iter().find(|i| i.id == registered_audio.id).expect("Audio item missing after restart");
    assert_eq!(restarted_audio_item.format, "MP3");

    let restarted_pl_item = persisted_items.iter().find(|i| i.id == registered_pl_item.id).expect("Playlist item missing after restart");
    assert_eq!(restarted_pl_item.source_playlist_id, Some(playlist_info.id));

    let restarted_missing_item = persisted_items.iter().find(|i| i.id == reg_missing.id).expect("Missing item missing after restart");
    assert_eq!(restarted_missing_item.file_status, "MISSING");

    println!("Verified 100% SQLite Restart Persistence Across App Restarts!");
    println!("\n=== ALL REAL-WORLD PHASE 5 VERIFICATION TESTS PASSED SUCCESSFULLY! ===");
}
