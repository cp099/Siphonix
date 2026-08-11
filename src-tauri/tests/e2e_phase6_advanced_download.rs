use std::fs;
use std::sync::Arc;
use tempfile::tempdir;
use chrono::Utc;

use siphonix_lib::db::DbRepository;
use siphonix_lib::engine::builder::{CommandBuilder, DownloadRequest};
use siphonix_lib::engine::detector::EngineDetector;
use siphonix_lib::engine::options::DownloadOptions;
use siphonix_lib::engine::runner::OutputParser;
use siphonix_lib::library::LibraryService;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::state::JobState;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::test]
async fn test_real_world_phase6_advanced_download_pipeline() {
    let engine = EngineDetector::detect().expect("yt-dlp and ffmpeg must be detected");
    let test_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

    let temp = tempdir().expect("tempdir failed");
    let db_path = temp.path().join("phase6_advanced.db");
    let dest_dir = temp.path().join("phase6_downloads");
    fs::create_dir_all(&dest_dir).unwrap();

    let db = DbRepository::init(&db_path).await.expect("DB init failed");
    let db_arc = Arc::new(db);
    let library_service = LibraryService::new(db_arc.clone());

    // 1. Build Phase 6 Advanced DownloadOptions
    let mut opts = DownloadOptions::default();
    opts.media_mode = "audio".to_string();
    opts.audio.format = "MP3".to_string();
    opts.audio.quality = "320k".to_string();
    opts.output.destination_path = dest_dir.to_string_lossy().to_string();
    opts.output.naming_preset = "custom".to_string();
    opts.output.custom_naming_template = Some("%(title)s [%(id)s].%(ext)s".to_string());
    opts.metadata.embed_metadata = true;
    opts.metadata.embed_thumbnail = true;

    // Validate options
    opts.validate().expect("DownloadOptions validation failed");

    // 2. Create DownloadJob with options snapshot
    let job_id = format!("job-p6-adv-{}", Utc::now().timestamp_millis());
    let job = DownloadJob {
        id: job_id.clone(),
        url: test_url.to_string(),
        title: "Big Buck Bunny 60fps 4K".to_string(),
        thumbnail_url: Some("https://i.ytimg.com/vi/aqz-KE-bpKQ/hqdefault.jpg".to_string()),
        media_mode: "audio".to_string(),
        format: "MP3".to_string(),
        quality: "320k".to_string(),
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
        started_at: Some(Utc::now()),
        completed_at: None,
        source_video_id: Some("aqz-KE-bpKQ".to_string()),
        source_playlist_id: None,
        source_playlist_title: None,
        playlist_entry_index: None,
        options: opts.clone(),
    };

    // Insert job into database
    db_arc.insert_job(&job.to_db_job()).await.expect("Failed to insert job into DB");

    // 3. Build Command via CommandBuilder consuming options snapshot
    let req = DownloadRequest {
        url: test_url.to_string(),
        media_mode: "audio".to_string(),
        audio_format: Some("MP3".to_string()),
        audio_quality: Some("320k".to_string()),
        video_format: None,
        video_quality: None,
        destination_path: dest_dir.to_string_lossy().to_string(),
        options: Some(opts.clone()),
    };

    let mut args = CommandBuilder::build_args(&req, &engine);
    if std::env::var("SIPHONIX_DEV_INSECURE_SSL").is_ok() {
        args.push("--no-check-certificates".to_string());
    }

    println!("CommandBuilder CLI args: {:?}", args);

    // Verify command flags emitted from options
    assert!(args.contains(&"--add-metadata".to_string()), "Must emit --add-metadata flag");
    assert!(args.contains(&"--embed-thumbnail".to_string()), "Must emit --embed-thumbnail flag");
    assert!(args.contains(&"-o".to_string()), "Must emit -o output template flag");

    // 4. Run process with OutputParser filepath tracking
    let mut cmd = tokio::process::Command::new(&engine.yt_dlp);
    cmd.args(&args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn yt-dlp download");
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();
    let mut captured_path: Option<String> = None;

    while let Ok(Some(line)) = reader.next_line().await {
        if OutputParser::is_filepath_line(&line) {
            captured_path = Some(line.trim().to_string());
        }
    }

    let status = child.wait().await.expect("Process wait failed");
    assert!(status.success(), "Real advanced download process failed");
    assert!(captured_path.is_some(), "after_move:filepath line must be captured");

    let final_path_str = captured_path.unwrap();
    let final_path = std::path::Path::new(&final_path_str);
    println!("Captured Advanced Final Output Path: {}", final_path_str);

    // 5. Verification Checklist:
    // Criteria 1: Download completes successfully.
    // Criteria 2: Final file exists on disk.
    assert!(final_path.exists(), "Final file must exist on disk");

    // Criteria 3: Filename follows custom naming template %(title)s [%(id)s].%(ext)s
    let file_name = final_path.file_name().unwrap().to_str().unwrap();
    assert!(file_name.contains("[aqz-KE-bpKQ]"), "Filename must contain YouTube ID in brackets matching custom template");
    assert!(file_name.ends_with(".mp3"), "Filename must end with .mp3");

    // Criteria 4: File size is valid
    let metadata = fs::metadata(final_path).expect("File metadata failed");
    assert!(metadata.len() > 10_000, "Output MP3 size must be non-zero");

    // Criteria 6: Register completed job in Library
    let library_item = library_service
        .register_completed_job(&job, Some(&final_path_str))
        .await
        .expect("Failed to register item in library");

    assert_eq!(library_item.file_extension, "mp3");
    assert_eq!(library_item.format, "MP3");

    // Criteria 7 & 8: Verify options_json snapshot persistence in jobs and library_items
    let loaded_db_job = db_arc.get_all_jobs().await.unwrap().into_iter().find(|j| j.id == job_id).unwrap();
    let loaded_job = DownloadJob::from_db_job(loaded_db_job);
    assert_eq!(loaded_job.options.metadata.embed_metadata, true);
    assert_eq!(loaded_job.options.metadata.embed_thumbnail, true);

    let loaded_db_item = db_arc.get_library_item_by_id(&library_item.id).await.unwrap().unwrap();
    let loaded_item_opts: DownloadOptions = serde_json::from_str(&loaded_db_item.options_json.unwrap()).unwrap();
    assert_eq!(loaded_item_opts.output.naming_preset, "custom");
    assert_eq!(loaded_item_opts.output.custom_naming_template.unwrap(), "%(title)s [%(id)s].%(ext)s");

    println!("=== REAL-WORLD PHASE 6 ADVANCED DOWNLOAD PIPELINE PASSED CLEANLY! ===");
}
