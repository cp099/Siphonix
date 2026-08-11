use std::sync::Arc;
use tempfile::tempdir;
use chrono::Utc;

use siphonix_lib::db::repository::{DbRepository, DbPreset};
use siphonix_lib::engine::builder::{CommandBuilder, DownloadRequest};
use siphonix_lib::engine::detector::EnginePaths;
use siphonix_lib::engine::options::{DownloadOptions, ValidationError};
use siphonix_lib::queue::job::DownloadJob;
use siphonix_lib::queue::state::JobState;

fn mock_engine_paths() -> EnginePaths {
    EnginePaths {
        yt_dlp: std::path::PathBuf::from("/usr/local/bin/yt-dlp"),
        ffmpeg: std::path::PathBuf::from("/usr/local/bin/ffmpeg"),
    }
}

#[tokio::test]
async fn test_options_single_source_of_truth() {
    let mut opts = DownloadOptions::default();
    opts.media_mode = "audio".to_string();
    opts.audio.format = "FLAC".to_string();
    opts.audio.quality = "320k".to_string(); // Bitrate meaningless for lossless

    opts.validate().expect("Validation failed");

    // Verify audio normalization
    assert_eq!(opts.audio.quality, "best");
    assert!(!opts.subtitles.enabled);
    assert!(!opts.subtitles.embed_in_video);
}

#[tokio::test]
async fn test_naming_template_syntax_regex() {
    // Valid formatted templates
    assert!(DownloadOptions::validate_template_string("%(title)s.%(ext)s").is_ok());
    assert!(DownloadOptions::validate_template_string("%(playlist_index)02d - %(title)s [%(id)s].%(ext)s").is_ok());
    assert!(DownloadOptions::validate_template_string("%(artist)s - %(title).100s.%(ext)s").is_ok());

    // Invalid templates
    assert!(DownloadOptions::validate_template_string("../%(title)s.%(ext)s").is_err());
    assert!(DownloadOptions::validate_template_string("sub/dir/%(title)s.%(ext)s").is_err());
    assert!(DownloadOptions::validate_template_string("%(title)s; rm -rf /").is_err());
    assert!(DownloadOptions::validate_template_string("%(title)s | grep test").is_err());
    assert!(DownloadOptions::validate_template_string("%(title)s<invalid>").is_err());
}

#[tokio::test]
async fn test_subtitle_container_incompatibility() {
    let mut opts = DownloadOptions::default();
    opts.media_mode = "video".to_string();
    opts.output.container = "MP4".to_string();
    opts.subtitles.enabled = true;
    opts.subtitles.format = "ass".to_string();
    opts.subtitles.embed_in_video = true;

    let res = opts.validate();
    assert!(res.is_err());
    match res.unwrap_err() {
        ValidationError::SubtitleContainerIncompatibility(msg) => {
            assert!(msg.contains("MP4 container cannot embed ASS subtitles natively"));
        }
        other => panic!("Unexpected error type: {:?}", other),
    }

    // Verify user container selection is NOT silently mutated to MKV
    assert_eq!(opts.output.container, "MP4");
}

#[tokio::test]
async fn test_command_builder_format_sort_preferences() {
    // 1. Prefer 60 FPS
    let mut opts_fps60 = DownloadOptions::default();
    opts_fps60.video.frame_rate = "60".to_string();
    opts_fps60.video.selection_mode = "prefer".to_string();

    let req_fps60 = DownloadRequest {
        url: "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string(),
        media_mode: "video".to_string(),
        audio_format: None,
        audio_quality: None,
        video_format: Some("MP4".to_string()),
        video_quality: Some("1080p".to_string()),
        destination_path: "/Downloads".to_string(),
        options: Some(opts_fps60),
    };
    let args_fps60 = CommandBuilder::build_args(&req_fps60, &mock_engine_paths());
    let sort_idx = args_fps60.iter().position(|r| r == "-S").unwrap();
    assert!(args_fps60[sort_idx + 1].contains("fps:60"));

    // 2. Prefer HDR
    let mut opts_hdr = DownloadOptions::default();
    opts_hdr.video.hdr_preference = "hdr".to_string();

    let req_hdr = DownloadRequest {
        url: "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string(),
        media_mode: "video".to_string(),
        audio_format: None,
        audio_quality: None,
        video_format: Some("MP4".to_string()),
        video_quality: Some("1080p".to_string()),
        destination_path: "/Downloads".to_string(),
        options: Some(opts_hdr),
    };
    let args_hdr = CommandBuilder::build_args(&req_hdr, &mock_engine_paths());
    let sort_hdr = args_hdr.iter().position(|r| r == "-S").unwrap();
    assert!(args_hdr[sort_hdr + 1].starts_with("hdr,"));

    // 3. Prefer SDR (+hdr prefix)
    let mut opts_sdr = DownloadOptions::default();
    opts_sdr.video.hdr_preference = "sdr".to_string();

    let req_sdr = DownloadRequest {
        url: "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string(),
        media_mode: "video".to_string(),
        audio_format: None,
        audio_quality: None,
        video_format: Some("MP4".to_string()),
        video_quality: Some("1080p".to_string()),
        destination_path: "/Downloads".to_string(),
        options: Some(opts_sdr),
    };
    let args_sdr = CommandBuilder::build_args(&req_sdr, &mock_engine_paths());
    let sort_sdr = args_sdr.iter().position(|r| r == "-S").unwrap();
    assert!(args_sdr[sort_sdr + 1].starts_with("+hdr,"));
}

#[tokio::test]
async fn test_preset_single_default_transaction() {
    let dir = tempdir().expect("Failed tempdir");
    let db_path = dir.path().join("presets_test.db");

    let db = DbRepository::init(&db_path).await.expect("DB init failed");

    let opts = DownloadOptions::default();
    let opts_json = serde_json::to_string(&opts).unwrap();

    let p1 = DbPreset {
        id: "preset-1".to_string(),
        name: "Preset 1".to_string(),
        description: None,
        is_default: 1,
        options_json: opts_json.clone(),
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };

    let p2 = DbPreset {
        id: "preset-2".to_string(),
        name: "Preset 2".to_string(),
        description: None,
        is_default: 0,
        options_json: opts_json.clone(),
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };

    db.insert_preset(&p1).await.unwrap();
    db.insert_preset(&p2).await.unwrap();

    // Set Preset 2 as default -> Preset 1 is_default must become 0
    db.set_default_preset("preset-2").await.unwrap();

    let p1_updated = db.get_preset_by_id("preset-1").await.unwrap().unwrap();
    let p2_updated = db.get_preset_by_id("preset-2").await.unwrap().unwrap();

    assert_eq!(p1_updated.is_default, 0);
    assert_eq!(p2_updated.is_default, 1);
}

#[tokio::test]
async fn test_job_and_library_options_snapshot() {
    let dir = tempdir().expect("Failed tempdir");
    let db_path = dir.path().join("snapshot_test.db");

    let db = DbRepository::init(&db_path).await.expect("DB init failed");

    let mut custom_opts = DownloadOptions::default();
    custom_opts.video.resolution = "2160p".to_string();
    custom_opts.video.codec_preference = "av1".to_string();

    let job = DownloadJob {
        id: "job-snap-1".to_string(),
        url: "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string(),
        title: "Test Snapshot".to_string(),
        thumbnail_url: None,
        media_mode: "video".to_string(),
        format: "MP4".to_string(),
        quality: "2160p".to_string(),
        destination_path: dir.path().to_string_lossy().to_string(),
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
        options: custom_opts.clone(),
    };

    db.insert_job(&job.to_db_job()).await.unwrap();

    let loaded_db_job = db.get_all_jobs().await.unwrap().into_iter().find(|j| j.id == "job-snap-1").unwrap();
    let loaded_job = DownloadJob::from_db_job(loaded_db_job);

    assert_eq!(loaded_job.options.video.resolution, "2160p");
    assert_eq!(loaded_job.options.video.codec_preference, "av1");
}
