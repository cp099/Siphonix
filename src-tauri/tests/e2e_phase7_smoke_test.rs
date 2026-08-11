use std::sync::Arc;
use tempfile::tempdir;

use siphonix_lib::db::DbRepository;
use siphonix_lib::engine::builder::{CommandBuilder, DownloadRequest};
use siphonix_lib::engine::options::{AudioOptions, DownloadOptions, MetadataOptions, OutputOptions};
use siphonix_lib::engine::runner::OutputParser;
use siphonix_lib::engine::DownloadManager;
use siphonix_lib::library::LibraryService;
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::state::JobState;
use siphonix_lib::runtime::{EngineManager, EngineStatusState};

#[tokio::test]
async fn test_real_world_phase7_native_smoke_test() {
    let tmp = tempdir().unwrap();
    let app_data_dir = tmp.path().join("app_data");
    let download_dir = tmp.path().join("downloads");
    std::fs::create_dir_all(&app_data_dir).unwrap();
    std::fs::create_dir_all(&download_dir).unwrap();

    let db_path = app_data_dir.join("siphonix_phase7.db");

    println!("=== Phase 7 Real-World Native Smoke Test ===");
    println!("Database Path: {:?}", db_path);
    println!("Download Destination: {:?}", download_dir);

    // 1. Initialize EngineManager with Production Resolution Policy
    let engine_manager = Arc::new(EngineManager::new(&app_data_dir, Some(true)));
    let initial_status = engine_manager.refresh_status().await;

    println!("--- 2. Runtime Status Verification ---");
    println!("Runtime Overall Ready: {}", initial_status.ready);
    println!("yt-dlp Name: {}", initial_status.yt_dlp.name);
    println!("yt-dlp Status: {:?}", initial_status.yt_dlp.status);
    println!("yt-dlp Version: {:?}", initial_status.yt_dlp.version);
    println!("yt-dlp Source: {:?}", initial_status.yt_dlp.source);
    println!("yt-dlp Path: {:?}", initial_status.yt_dlp.path);

    println!("FFmpeg Name: {}", initial_status.ffmpeg.name);
    println!("FFmpeg Status: {:?}", initial_status.ffmpeg.status);
    println!("FFmpeg Version: {:?}", initial_status.ffmpeg.version);
    println!("FFmpeg Source: {:?}", initial_status.ffmpeg.source);
    println!("FFmpeg Path: {:?}", initial_status.ffmpeg.path);

    assert!(
        initial_status.yt_dlp.status == EngineStatusState::Ready,
        "yt-dlp engine must be READY for real-world smoke test"
    );
    assert!(
        initial_status.ffmpeg.status == EngineStatusState::Ready,
        "FFmpeg engine must be READY for real-world smoke test"
    );

    // 2. Initialize DownloadManager with EngineManager integration
    let download_manager = DownloadManager::new_with_runtime(engine_manager.clone());
    let resolved_paths = download_manager.ensure_engine().await.unwrap();

    println!("EnginePaths resolved via EngineManager:");
    println!("  yt-dlp: {:?}", resolved_paths.yt_dlp);
    println!("  ffmpeg: {:?}", resolved_paths.ffmpeg);

    // 3. Configure Phase 6 Advanced Options
    let mut options = DownloadOptions::default();
    options.media_mode = "audio".to_string();
    options.audio = AudioOptions {
        format: "mp3".to_string(),
        quality: "320k".to_string(),
        codec_preference: "auto".to_string(),
    };
    options.metadata = MetadataOptions {
        embed_metadata: true,
        embed_thumbnail: true,
        write_metadata_json: false,
    };
    options.output = OutputOptions {
        container: "mp3".to_string(),
        naming_preset: "custom".to_string(),
        custom_naming_template: Some("%(title)s [%(id)s].%(ext)s".to_string()),
        destination_path: download_dir.to_string_lossy().to_string(),
        folder_organization: "flat".to_string(),
        overwrite_policy: "never".to_string(),
    };

    let test_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string();
    let job_id = "phase7-smoke-job-1".to_string();

    let job = DownloadJob {
        id: job_id.clone(),
        url: test_url.clone(),
        title: "Big Buck Bunny 60fps 4K".to_string(),
        thumbnail_url: None,
        media_mode: "audio".to_string(),
        format: "mp3".to_string(),
        quality: "320k".to_string(),
        destination_path: download_dir.to_string_lossy().to_string(),
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
        options: options.clone(),
    };

    // Init SQLite database
    let db = Arc::new(DbRepository::init(&db_path).await.unwrap());
    let db_job = job.to_db_job();
    db.insert_job(&db_job).await.unwrap();

    let request = DownloadRequest {
        url: test_url,
        media_mode: "audio".to_string(),
        audio_format: Some("mp3".to_string()),
        audio_quality: Some("320k".to_string()),
        video_format: None,
        video_quality: None,
        destination_path: download_dir.to_string_lossy().to_string(),
        options: Some(options.clone()),
    };

    println!("--- 3. Running Real Download via EngineManager-Resolved Executables ---");

    // Spawn process directly via EngineManager engine paths
    let mut cmd = tokio::process::Command::new(&resolved_paths.yt_dlp);
    let args = CommandBuilder::build_args(&request, &resolved_paths);
    cmd.args(&args);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    println!("Executing Command directly without shell wrapper: {:?}", cmd);

    let mut child = cmd.spawn().expect("Failed to launch yt-dlp binary");
    let stdout = child.stdout.take().unwrap();
    let mut reader = tokio::io::BufReader::new(stdout);
    use tokio::io::AsyncBufReadExt;

    let mut captured_path: Option<String> = None;
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if OutputParser::is_filepath_line(&line) {
            captured_path = Some(line.trim().to_string());
        }
    }

    let status = child.wait().await.unwrap();
    assert!(status.success(), "yt-dlp process must exit with success status");

    let final_file_path = captured_path.expect("Must capture final output file path from output stream");
    println!("Captured Advanced Final Output Path: {}", final_file_path);

    // 4. Verify Output File on Disk
    let output_file = std::path::PathBuf::from(&final_file_path);
    assert!(output_file.exists(), "Output file must exist on disk");
    let metadata = std::fs::metadata(&output_file).unwrap();
    assert!(metadata.len() > 10000, "Output file size must be non-zero (> 10 KB)");
    assert!(
        output_file.file_name().unwrap().to_string_lossy().contains("[aqz-KE-bpKQ]"),
        "Filename must match custom template containing ID in brackets"
    );
    assert!(
        final_file_path.ends_with(".mp3"),
        "File extension must be .mp3"
    );

    println!("--- 5. Library Registration & Options Snapshot Verification ---");
    let mut completed_job = job.clone();
    completed_job.state = JobState::COMPLETED;
    completed_job.completed_at = Some(chrono::Utc::now());
    let completed_at_str = completed_job.completed_at.unwrap().to_rfc3339();

    db.update_job_state(&job_id, "COMPLETED", None, None, 0, None, Some(&completed_at_str))
        .await
        .unwrap();

    let lib_service = LibraryService::new(db.clone());
    let lib_item = lib_service
        .register_completed_job(&completed_job, Some(&final_file_path))
        .await
        .unwrap();

    assert_eq!(lib_item.file_status, "AVAILABLE");
    assert_eq!(lib_item.format.to_uppercase(), "MP3");
    assert!(lib_item.options_json.is_some());

    let all_jobs = db.get_all_jobs().await.unwrap();
    let stored_job = all_jobs.iter().find(|j| j.id == job_id).expect("Job must exist in DB");
    assert!(!stored_job.options_json.is_empty());

    println!("--- 6. Re-verifying Runtime Status Post-Download ---");
    let post_status = engine_manager.get_status().await;
    assert!(post_status.ready);
    assert_eq!(post_status.yt_dlp.status, EngineStatusState::Ready);
    assert_eq!(post_status.ffmpeg.status, EngineStatusState::Ready);

    println!("=== PHASE 7 REAL-WORLD NATIVE SMOKE TEST PASSED CLEANLY! ===");
}
