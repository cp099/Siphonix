use std::path::Path;
use serde::{Deserialize, Serialize};
use super::detector::EnginePaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    pub media_mode: String, // "audio" | "video"
    pub audio_format: Option<String>, // "MP3" | "M4A" | "AAC" | "FLAC" | "ALAC" | "OPUS" | "WAV"
    pub audio_quality: Option<String>, // "best" | "320k" | "256k" | "192k" | "128k"
    pub video_format: Option<String>, // "MP4" | "MKV" | "WEBM"
    pub video_quality: Option<String>, // "best" | "2160p" | "1440p" | "1080p" | "720p" | "480p" | "360p"
    pub destination_path: String,
}

pub struct CommandBuilder;

impl CommandBuilder {
    pub fn build_args(request: &DownloadRequest, engine: &EnginePaths) -> Vec<String> {
        let mut args = Vec::new();

        // 1. General & Telemetry flags
        args.push("--newline".to_string());
        args.push("--no-playlist".to_string());
        args.push("--progress-template".to_string());
        args.push("SIPHONIX_PROGRESS|%(progress.status)s|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(progress._total_bytes_str)s".to_string());

        // 2. FFmpeg location flag if available
        if let Some(ffmpeg_dir) = engine.ffmpeg.parent() {
            args.push("--ffmpeg-location".to_string());
            args.push(ffmpeg_dir.to_string_lossy().to_string());
        }

        // 3. Media Mode (Video vs Audio)
        if request.media_mode == "audio" {
            Self::build_audio_args(&mut args, request);
        } else {
            Self::build_video_args(&mut args, request);
        }

        // 4. Output Path Template
        let dest_dir = Path::new(&request.destination_path);
        let output_template = dest_dir.join("%(title)s.%(ext)s");
        args.push("-o".to_string());
        args.push(output_template.to_string_lossy().to_string());

        // 5. Target URL
        args.push(request.url.clone());

        args
    }

    fn build_video_args(args: &mut Vec<String>, request: &DownloadRequest) {
        let quality = request.video_quality.as_deref().unwrap_or("best");

        let format_selector = match quality {
            "2160p" => "bv*[height<=2160]+ba/b[height<=2160]/wv*+ba/w",
            "1440p" => "bv*[height<=1440]+ba/b[height<=1440]/wv*+ba/w",
            "1080p" => "bv*[height<=1080]+ba/b[height<=1080]/wv*+ba/w",
            "720p"  => "bv*[height<=720]+ba/b[height<=720]/wv*+ba/w",
            "480p"  => "bv*[height<=480]+ba/b[height<=480]/wv*+ba/w",
            "360p"  => "bv*[height<=360]+ba/b[height<=360]/wv*+ba/w",
            _       => "bv*+ba/b",
        };

        args.push("-f".to_string());
        args.push(format_selector.to_string());

        if let Some(container) = &request.video_format {
            let container_lower = container.to_lowercase();
            args.push("--merge-output-format".to_string());
            args.push(container_lower);
        }
    }

    fn build_audio_args(args: &mut Vec<String>, request: &DownloadRequest) {
        let fmt = request.audio_format.as_deref().unwrap_or("MP3").to_lowercase();
        let quality = request.audio_quality.as_deref().unwrap_or("best");

        // Audio stream extraction
        args.push("-x".to_string());

        match fmt.as_str() {
            "m4a" | "aac" | "opus" | "alac" => {
                // Stream-smart extraction: prefer native stream extraction without re-encoding
                args.push("--audio-format".to_string());
                args.push(fmt);
            }
            _ => {
                // MP3, FLAC, WAV - require FFmpeg conversion/re-encoding
                args.push("--audio-format".to_string());
                args.push(fmt);

                let q_val = match quality {
                    "320k" => "320k",
                    "256k" => "256k",
                    "192k" => "192k",
                    "128k" => "128k",
                    _ => "0", // 0 is best VBR quality for ffmpeg in yt-dlp
                };
                args.push("--audio-quality".to_string());
                args.push(q_val.to_string());
            }
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
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"bv*+ba/b".to_string()));
        assert!(args.contains(&"--merge-output-format".to_string()));
        assert!(args.contains(&"mp4".to_string()));
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
        };

        let args = CommandBuilder::build_args(&req, &mock_engine_paths());
        assert!(args.contains(&"--ffmpeg-location".to_string()));
        assert!(args.contains(&"/usr/local/bin".to_string()));
    }
}
