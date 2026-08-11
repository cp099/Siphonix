use std::path::Path;
use serde::{Deserialize, Serialize};
use super::detector::EnginePaths;
use super::options::DownloadOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    pub media_mode: String, // "audio" | "video"
    pub audio_format: Option<String>,
    pub audio_quality: Option<String>,
    pub video_format: Option<String>,
    pub video_quality: Option<String>,
    pub destination_path: String,
    pub options: Option<DownloadOptions>,
}

pub struct CommandBuilder;

impl CommandBuilder {
    pub fn build_args(request: &DownloadRequest, engine: &EnginePaths) -> Vec<String> {
        let mut args = Vec::new();

        let opts = request.options.clone().unwrap_or_else(|| {
            let mut d = DownloadOptions::default();
            d.media_mode = request.media_mode.clone();
            d.output.destination_path = request.destination_path.clone();
            if let Some(ref vf) = request.video_format {
                d.output.container = vf.clone();
            }
            if let Some(ref vq) = request.video_quality {
                d.video.resolution = vq.clone();
            }
            if let Some(ref af) = request.audio_format {
                d.audio.format = af.clone();
            }
            if let Some(ref aq) = request.audio_quality {
                d.audio.quality = aq.clone();
            }
            d
        });

        // 1. General & Telemetry flags
        args.push("--newline".to_string());
        if std::env::var("SIPHONIX_DEV_INSECURE_SSL").is_ok() {
            args.push("--no-check-certificates".to_string());
        }
        args.push("--no-playlist".to_string());
        args.push("--print".to_string());
        args.push("after_move:filepath".to_string());
        args.push("--progress-template".to_string());
        args.push("SIPHONIX_PROGRESS|%(progress.status)s|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(progress._total_bytes_str)s".to_string());

        // 2. FFmpeg location flag if available
        if let Some(ffmpeg_dir) = engine.ffmpeg.parent() {
            args.push("--ffmpeg-location".to_string());
            args.push(ffmpeg_dir.to_string_lossy().to_string());
        }

        // 3. Format Sort Strategy & Codec / FPS / HDR Preferences
        Self::build_format_sort_args(&mut args, &opts);

        // 4. Overwrite Policy
        match opts.output.overwrite_policy.as_str() {
            "never" => args.push("--no-overwrites".to_string()),
            "replace" => args.push("--force-overwrites".to_string()),
            _ => {}
        }

        // 5. Metadata, Thumbnails & Info JSON
        if opts.metadata.embed_metadata {
            args.push("--add-metadata".to_string());
        }
        if opts.metadata.embed_thumbnail {
            args.push("--embed-thumbnail".to_string());
        }
        if opts.metadata.write_metadata_json {
            args.push("--write-info-json".to_string());
        }

        // 6. Subtitles
        if opts.subtitles.enabled && opts.media_mode == "video" {
            args.push("--write-subs".to_string());
            if !opts.subtitles.languages.is_empty() {
                args.push("--sub-langs".to_string());
                args.push(opts.subtitles.languages.join(","));
            }
            if opts.subtitles.format != "auto" {
                args.push("--convert-subs".to_string());
                args.push(opts.subtitles.format.to_lowercase());
            }
            if opts.subtitles.embed_in_video {
                args.push("--embed-subs".to_string());
            }
        }

        // 7. Network / Fragment Concurrency & Rate Limit
        if opts.network.concurrent_fragments != "auto" {
            if let Ok(n) = opts.network.concurrent_fragments.parse::<u32>() {
                args.push("--concurrent-fragments".to_string());
                args.push(n.to_string());
            }
        }
        if let Some(ref limit) = opts.network.max_download_rate {
            if !limit.trim().is_empty() {
                args.push("--limit-rate".to_string());
                args.push(limit.clone());
            }
        }

        // 8. Media Mode (Video vs Audio)
        if opts.media_mode == "audio" {
            Self::build_audio_args(&mut args, &opts);
        } else {
            Self::build_video_args(&mut args, &opts);
        }

        // 9. Output Path Template
        let dest_dir = Path::new(&opts.output.destination_path);
        let template_rel = opts.get_effective_output_template();
        let output_template = dest_dir.join(template_rel);
        args.push("-o".to_string());
        args.push(output_template.to_string_lossy().to_string());

        // 10. Target URL
        args.push(request.url.clone());

        args
    }

    fn build_format_sort_args(args: &mut Vec<String>, opts: &DownloadOptions) {
        if opts.media_mode == "audio" {
            return;
        }

        let mut sort_tokens = Vec::new();

        // HDR preference
        match opts.video.hdr_preference.as_str() {
            "hdr" => sort_tokens.push("hdr".to_string()),
            "sdr" => sort_tokens.push("+hdr".to_string()),
            _ => {}
        }

        // Codec preference (prefer mode)
        if opts.video.selection_mode == "prefer" {
            match opts.video.codec_preference.as_str() {
                "av1" => sort_tokens.push("vcodec:av01".to_string()),
                "vp9" => sort_tokens.push("vcodec:vp9".to_string()),
                "h264" => sort_tokens.push("vcodec:h264".to_string()),
                _ => {}
            }
        }

        // FPS preference (prefer mode)
        if opts.video.selection_mode == "prefer" {
            match opts.video.frame_rate.as_str() {
                "60" => sort_tokens.push("fps:60".to_string()),
                "30" => sort_tokens.push("fps:30".to_string()),
                "24" => sort_tokens.push("fps:24".to_string()),
                _ => {}
            }
        }

        // Base strategy fallback
        match opts.expert.format_sort_strategy.as_str() {
            "codec_first" => sort_tokens.extend(["vcodec", "res", "fps", "acodec"].map(String::from)),
            "fps_first" => sort_tokens.extend(["fps", "res", "vcodec", "acodec"].map(String::from)),
            "size_first" => sort_tokens.extend(["size", "res", "vcodec", "acodec"].map(String::from)),
            "audio_first" => sort_tokens.extend(["acodec", "abr", "res", "vcodec"].map(String::from)),
            _ => sort_tokens.extend(["res", "vcodec", "fps", "acodec"].map(String::from)), // "resolution_first"
        }

        args.push("-S".to_string());
        args.push(sort_tokens.join(","));
    }

    fn build_video_args(args: &mut Vec<String>, opts: &DownloadOptions) {
        let quality = opts.video.resolution.as_str();

        let height_filter = match quality {
            "2160p" => "[height<=2160]",
            "1440p" => "[height<=1440]",
            "1080p" => "[height<=1080]",
            "720p"  => "[height<=720]",
            "480p"  => "[height<=480]",
            "360p"  => "[height<=360]",
            _       => "",
        };

        let mut req_filters = String::new();
        if opts.video.selection_mode == "require" {
            if opts.video.frame_rate != "auto" {
                req_filters.push_str(&format!("[fps<={}]", opts.video.frame_rate));
            }
            match opts.video.codec_preference.as_str() {
                "av1" => req_filters.push_str("[vcodec^=av01]"),
                "vp9" => req_filters.push_str("[vcodec^=vp9]"),
                "h264" => req_filters.push_str("[vcodec^=avc1]"),
                _ => {}
            }
        }

        let format_selector = if height_filter.is_empty() && req_filters.is_empty() {
            "bv*+ba/b".to_string()
        } else {
            format!("bv*{}{}+ba/b{}{}/wv*+ba/w", height_filter, req_filters, height_filter, req_filters)
        };

        args.push("-f".to_string());
        args.push(format_selector);

        let container_lower = opts.output.container.to_lowercase();
        args.push("--merge-output-format".to_string());
        args.push(container_lower);
    }

    fn build_audio_args(args: &mut Vec<String>, opts: &DownloadOptions) {
        let fmt = opts.audio.format.to_lowercase();
        let quality = opts.audio.quality.as_str();

        args.push("-x".to_string());
        args.push("--audio-format".to_string());
        args.push(fmt.clone());

        match fmt.as_str() {
            "mp3" | "flac" | "wav" => {
                let q_val = match quality {
                    "320k" => "320k",
                    "256k" => "256k",
                    "192k" => "192k",
                    "128k" => "128k",
                    _ => "0",
                };
                args.push("--audio-quality".to_string());
                args.push(q_val.to_string());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mock_engine_paths() -> EnginePaths {
        EnginePaths {
            yt_dlp: PathBuf::from("/usr/local/bin/yt-dlp"),
            ffmpeg: PathBuf::from("/usr/local/bin/ffmpeg"),
        }
    }

    #[test]
    fn test_builder_audio_mp3_320k() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "audio".to_string(),
            audio_format: Some("MP3".to_string()),
            audio_quality: Some("320k".to_string()),
            video_format: None,
            video_quality: None,
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"-x".to_string()));
        assert!(args.contains(&"mp3".to_string()));
        assert!(args.contains(&"320k".to_string()));
        assert_eq!(args.last().unwrap(), "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_builder_audio_flac() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "audio".to_string(),
            audio_format: Some("FLAC".to_string()),
            audio_quality: Some("best".to_string()),
            video_format: None,
            video_quality: None,
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"-x".to_string()));
        assert!(args.contains(&"flac".to_string()));
    }

    #[test]
    fn test_builder_video_best() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("best".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"bv*+ba/b".to_string()));
        assert!(args.contains(&"--merge-output-format".to_string()));
        assert!(args.contains(&"mp4".to_string()));
    }

    #[test]
    fn test_builder_fps_prefer_vs_require() {
        let mut opts_prefer = DownloadOptions::default();
        opts_prefer.video.frame_rate = "60".to_string();
        opts_prefer.video.selection_mode = "prefer".to_string();

        let req_prefer = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("1080p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: Some(opts_prefer),
        };

        let args_prefer = CommandBuilder::build_args(&req_prefer, &mock_engine_paths());
        let sort_idx = args_prefer.iter().position(|r| r == "-S").unwrap();
        assert!(args_prefer[sort_idx + 1].contains("fps:60"));

        let mut opts_require = DownloadOptions::default();
        opts_require.video.frame_rate = "60".to_string();
        opts_require.video.selection_mode = "require".to_string();

        let req_require = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("1080p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: Some(opts_require),
        };

        let args_require = CommandBuilder::build_args(&req_require, &mock_engine_paths());
        let format_idx = args_require.iter().position(|r| r == "-f").unwrap();
        assert!(args_require[format_idx + 1].contains("[fps<=60]"));
    }

    #[test]
    fn test_builder_hdr_and_sdr_preferences() {
        let mut opts_hdr = DownloadOptions::default();
        opts_hdr.video.hdr_preference = "hdr".to_string();

        let req_hdr = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("1080p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: Some(opts_hdr),
        };

        let args_hdr = CommandBuilder::build_args(&req_hdr, &mock_engine_paths());
        let sort_hdr = args_hdr.iter().position(|r| r == "-S").unwrap();
        assert!(args_hdr[sort_hdr + 1].starts_with("hdr,"));

        let mut opts_sdr = DownloadOptions::default();
        opts_sdr.video.hdr_preference = "sdr".to_string();

        let req_sdr = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("1080p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: Some(opts_sdr),
        };

        let args_sdr = CommandBuilder::build_args(&req_sdr, &mock_engine_paths());
        let sort_sdr = args_sdr.iter().position(|r| r == "-S").unwrap();
        assert!(args_sdr[sort_sdr + 1].starts_with("+hdr,"));
    }

    #[test]
    fn test_builder_video_2160p() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("2160p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"bv*[height<=2160]+ba/b[height<=2160]/wv*+ba/w".to_string()));
    }

    #[test]
    fn test_builder_video_1080p() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MKV".to_string()),
            video_quality: Some("1080p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"bv*[height<=1080]+ba/b[height<=1080]/wv*+ba/w".to_string()));
        assert!(args.contains(&"mkv".to_string()));
    }

    #[test]
    fn test_builder_video_1440p() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("1440p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"bv*[height<=1440]+ba/b[height<=1440]/wv*+ba/w".to_string()));
    }

    #[test]
    fn test_builder_video_720p() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("720p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"bv*[height<=720]+ba/b[height<=720]/wv*+ba/w".to_string()));
    }

    #[test]
    fn test_builder_video_480p() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("480p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"bv*[height<=480]+ba/b[height<=480]/wv*+ba/w".to_string()));
    }

    #[test]
    fn test_builder_video_360p() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: Some("MP4".to_string()),
            video_quality: Some("360p".to_string()),
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"bv*[height<=360]+ba/b[height<=360]/wv*+ba/w".to_string()));
    }

    #[test]
    fn test_builder_ffmpeg_location() {
        let req = DownloadRequest {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            media_mode: "video".to_string(),
            audio_format: None,
            audio_quality: None,
            video_format: None,
            video_quality: None,
            destination_path: "/Downloads/Siphonix".to_string(),
            options: None,
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"--ffmpeg-location".to_string()));
        assert!(args.contains(&"/usr/local/bin".to_string()));
    }
}
