use std::fs;
use std::time::Duration;
use siphonix_lib::engine::builder::{CommandBuilder, DownloadRequest};
use siphonix_lib::engine::detector::EngineDetector;
use siphonix_lib::engine::info::inspect_url;
use tokio::process::Command;

#[tokio::test]
async fn test_real_metadata_inspection() {
    let engine = EngineDetector::detect().expect("yt-dlp and ffmpeg must be detected");
    let test_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ"; // Blender open movie clip

    let info = inspect_url(test_url, &engine)
        .await
        .expect("inspect_url failed");

    assert_eq!(info.id, "aqz-KE-bpKQ");
    assert!(!info.title.is_empty(), "Title should not be empty");
    assert!(info.duration.is_some(), "Duration should be populated");
    println!("Metadata Inspection OK: Title='{}', Duration={:?}s", info.title, info.duration);
}

#[tokio::test]
async fn test_real_video_download_mp4_1080p() {
    let engine = EngineDetector::detect().expect("yt-dlp and ffmpeg must be detected");
    let test_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

    let dest_dir = std::env::temp_dir().join("siphonix_e2e_video");
    let _ = fs::create_dir_all(&dest_dir);

    let req = DownloadRequest {
        url: test_url.to_string(),
        media_mode: "video".to_string(),
        audio_format: None,
        audio_quality: None,
        video_format: Some("MP4".to_string()),
        video_quality: Some("1080p".to_string()),
        destination_path: dest_dir.to_string_lossy().to_string(),
    };

    let args = CommandBuilder::build_args(&req, &engine);
    let mut cmd = Command::new(&engine.yt_dlp);
    cmd.args(&args);

    let output = cmd.output().await.expect("Failed to execute yt-dlp video download");
    assert!(output.status.success(), "Video download failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify MP4 file exists in dest_dir
    let entries: Vec<_> = fs::read_dir(&dest_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "mp4"))
        .collect();

    assert!(!entries.is_empty(), "Resulting MP4 video file must exist");
    let file_path = entries[0].path();
    let metadata = fs::metadata(&file_path).expect("File metadata error");
    assert!(metadata.len() > 100_000, "Downloaded MP4 file size must be non-zero");

    println!("Real Video Download OK: File='{}', Size={} bytes", file_path.display(), metadata.len());

    // Clean up
    let _ = fs::remove_dir_all(&dest_dir);
}

#[tokio::test]
async fn test_real_audio_download_mp3_320k() {
    let engine = EngineDetector::detect().expect("yt-dlp and ffmpeg must be detected");
    let test_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

    let dest_dir = std::env::temp_dir().join("siphonix_e2e_audio");
    let _ = fs::create_dir_all(&dest_dir);

    let req = DownloadRequest {
        url: test_url.to_string(),
        media_mode: "audio".to_string(),
        audio_format: Some("MP3".to_string()),
        audio_quality: Some("320k".to_string()),
        video_format: None,
        video_quality: None,
        destination_path: dest_dir.to_string_lossy().to_string(),
    };

    let args = CommandBuilder::build_args(&req, &engine);
    let mut cmd = Command::new(&engine.yt_dlp);
    cmd.args(&args);

    let output = cmd.output().await.expect("Failed to execute yt-dlp audio download");
    assert!(output.status.success(), "Audio download failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify MP3 file exists in dest_dir
    let entries: Vec<_> = fs::read_dir(&dest_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "mp3"))
        .collect();

    assert!(!entries.is_empty(), "Resulting MP3 audio file must exist");
    let file_path = entries[0].path();
    let metadata = fs::metadata(&file_path).expect("File metadata error");
    assert!(metadata.len() > 10_000, "Downloaded MP3 file size must be non-zero");

    println!("Real Audio Download OK: File='{}', Size={} bytes", file_path.display(), metadata.len());

    // Clean up
    let _ = fs::remove_dir_all(&dest_dir);
}

#[tokio::test]
async fn test_real_cancellation_lifecycle() {
    let engine = EngineDetector::detect().expect("yt-dlp and ffmpeg must be detected");
    let test_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

    let dest_dir = std::env::temp_dir().join("siphonix_e2e_cancel");
    let _ = fs::create_dir_all(&dest_dir);

    let req = DownloadRequest {
        url: test_url.to_string(),
        media_mode: "video".to_string(),
        audio_format: None,
        audio_quality: None,
        video_format: Some("MP4".to_string()),
        video_quality: Some("best".to_string()),
        destination_path: dest_dir.to_string_lossy().to_string(),
    };

    let args = CommandBuilder::build_args(&req, &engine);
    let mut child = Command::new(&engine.yt_dlp)
        .args(&args)
        .spawn()
        .expect("Failed to spawn process");

    // Sleep briefly while download starts
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Send kill signal (simulating ProcessRegistry cancellation)
    let kill_result = child.start_kill();
    assert!(kill_result.is_ok(), "Kill signal sent successfully");

    let status = child.wait().await;
    assert!(status.is_ok(), "Process reaped cleanly after cancellation");

    println!("Real Cancellation OK: Process killed cleanly.");
    let _ = fs::remove_dir_all(&dest_dir);
}
