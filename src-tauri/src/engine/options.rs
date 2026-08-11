use serde::{Deserialize, Serialize};
use regex::Regex;
use std::sync::OnceLock;

static TEMPLATE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_template_regex() -> &'static Regex {
    TEMPLATE_REGEX.get_or_init(|| {
        Regex::new(r"^([^%]|%(%|\((title|id|artist|uploader|playlist|playlist_title|playlist_index|ext|upload_date|resolution|fps)\)(?:0?\d+d|\.\d+s|s)?))*$")
            .unwrap()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadOptions {
    pub media_mode: String, // "audio" | "video"
    pub video: VideoOptions,
    pub audio: AudioOptions,
    pub output: OutputOptions,
    pub metadata: MetadataOptions,
    pub subtitles: SubtitleOptions,
    pub network: NetworkOptions,
    pub expert: ExpertOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoOptions {
    pub resolution: String,       // "best" | "2160p" | "1440p" | "1080p" | "720p" | "480p" | "360p"
    pub frame_rate: String,       // "auto" | "60" | "30" | "24"
    pub codec_preference: String, // "auto" | "av1" | "vp9" | "h264"
    pub hdr_preference: String,   // "auto" | "sdr" | "hdr"
    pub selection_mode: String,   // "prefer" | "require"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioOptions {
    pub format: String,           // "MP3" | "M4A" | "AAC" | "FLAC" | "ALAC" | "OPUS" | "WAV"
    pub quality: String,          // "best" | "320k" | "256k" | "192k" | "128k"
    pub codec_preference: String, // "auto" | "aac" | "mp3" | "opus" | "flac" | "alac"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputOptions {
    pub container: String,                    // "MP4" | "MKV" | "WEBM" (for video) or audio format
    pub destination_path: String,             // Destination directory
    pub naming_preset: String,                // "simple" | "title_id" | "artist_title" | "playlist_index" | "custom"
    pub custom_naming_template: Option<String>, // e.g. "%(playlist_index)02d - %(title)s [%(id)s].%(ext)s"
    pub folder_organization: String,          // "flat" | "by_playlist" | "by_channel" | "playlist_index"
    pub overwrite_policy: String,             // "ask" | "never" | "replace"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataOptions {
    pub embed_metadata: bool,
    pub embed_thumbnail: bool,
    pub write_metadata_json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubtitleOptions {
    pub enabled: bool,
    pub languages: Vec<String>, // e.g. ["en", "es"] or ["original"]
    pub format: String,        // "auto" | "srt" | "vtt" | "ass"
    pub embed_in_video: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkOptions {
    pub concurrent_fragments: String,   // "auto" | "1" | "2" | "4" | "8"
    pub max_download_rate: Option<String>, // e.g. "10M" or None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpertOptions {
    pub format_sort_strategy: String, // "resolution_first" | "codec_first" | "fps_first" | "size_first" | "audio_first"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadPreset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub options: DownloadOptions,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            media_mode: "video".to_string(),
            video: VideoOptions {
                resolution: "1080p".to_string(),
                frame_rate: "auto".to_string(),
                codec_preference: "auto".to_string(),
                hdr_preference: "auto".to_string(),
                selection_mode: "prefer".to_string(),
            },
            audio: AudioOptions {
                format: "MP3".to_string(),
                quality: "best".to_string(),
                codec_preference: "auto".to_string(),
            },
            output: OutputOptions {
                container: "MP4".to_string(),
                destination_path: String::new(),
                naming_preset: "simple".to_string(),
                custom_naming_template: None,
                folder_organization: "flat".to_string(),
                overwrite_policy: "ask".to_string(),
            },
            metadata: MetadataOptions {
                embed_metadata: true,
                embed_thumbnail: true,
                write_metadata_json: false,
            },
            subtitles: SubtitleOptions {
                enabled: false,
                languages: vec!["en".to_string()],
                format: "srt".to_string(),
                embed_in_video: false,
            },
            network: NetworkOptions {
                concurrent_fragments: "auto".to_string(),
                max_download_rate: None,
            },
            expert: ExpertOptions {
                format_sort_strategy: "resolution_first".to_string(),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid naming template: {0}")]
    InvalidNamingTemplate(String),

    #[error("Incompatible subtitle & container configuration: {0}")]
    SubtitleContainerIncompatibility(String),

    #[error("Invalid path or directory component: {0}")]
    InvalidPath(String),
}

impl DownloadOptions {
    pub fn validate(&mut self) -> Result<(), ValidationError> {
        // 1. Audio Mode Normalization
        if self.media_mode == "audio" {
            self.subtitles.enabled = false;
            self.subtitles.embed_in_video = false;
        }

        // 2. Lossless Audio Bitrate Normalization
        if self.media_mode == "audio" {
            let fmt = self.audio.format.to_uppercase();
            if fmt == "FLAC" || fmt == "WAV" || fmt == "ALAC" {
                self.audio.quality = "best".to_string();
            }
        }

        // 3. Custom Naming Template Validation
        if self.output.naming_preset == "custom" {
            if let Some(ref template) = self.output.custom_naming_template {
                Self::validate_template_string(template)?;
            } else {
                return Err(ValidationError::InvalidNamingTemplate(
                    "Custom naming preset selected but no template provided".to_string(),
                ));
            }
        }

        // 4. Subtitle / Container Compatibility Matrix Check
        if self.media_mode == "video" && self.subtitles.enabled && self.subtitles.embed_in_video {
            let container = self.output.container.to_uppercase();
            let sub_fmt = self.subtitles.format.to_lowercase();

            if container == "MP4" && sub_fmt == "ass" {
                return Err(ValidationError::SubtitleContainerIncompatibility(
                    "MP4 container cannot embed ASS subtitles natively without conversion to SRT or switching container to MKV.".to_string(),
                ));
            }
            if container == "WEBM" && (sub_fmt == "ass" || sub_fmt == "srt") {
                return Err(ValidationError::SubtitleContainerIncompatibility(
                    "WebM container requires VTT subtitles when embedded or switching container to MKV.".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn validate_template_string(template: &str) -> Result<(), ValidationError> {
        let t = template.trim();
        if t.is_empty() {
            return Err(ValidationError::InvalidNamingTemplate("Template cannot be empty".to_string()));
        }

        // Reject path separators, relative traversal, shell operators
        if t.contains('/') || t.contains('\\') || t.contains("..") || t.starts_with('/') {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template cannot contain path separators or relative directory traversal".to_string(),
            ));
        }

        let forbidden_shell_chars = ['$', '|', ';', '&', '`', '>', '<'];
        if t.chars().any(|c| forbidden_shell_chars.contains(&c)) {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template contains forbidden shell control characters".to_string(),
            ));
        }

        // Windows reserved characters check
        let win_reserved = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        if t.chars().any(|c| win_reserved.contains(&c)) {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template contains Windows reserved characters".to_string(),
            ));
        }

        // Regex syntax verification for valid yt-dlp fields & modifiers
        let re = get_template_regex();
        if !re.is_match(t) {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template contains unsupported yt-dlp field syntax or format specifiers".to_string(),
            ));
        }

        Ok(())
    }

    pub fn get_effective_output_template(&self) -> String {
        let base_pattern = match self.output.naming_preset.as_str() {
            "title_id" => "%(title)s [%(id)s].%(ext)s",
            "artist_title" => "%(artist)s - %(title)s.%(ext)s",
            "playlist_index" => "%(playlist_index)02d - %(title)s.%(ext)s",
            "custom" => self
                .output
                .custom_naming_template
                .as_deref()
                .unwrap_or("%(title)s.%(ext)s"),
            _ => "%(title)s.%(ext)s", // "simple"
        };

        match self.output.folder_organization.as_str() {
            "by_playlist" => format!("%(playlist_title|Playlists)s/{}", base_pattern),
            "by_channel" => format!("%(uploader|Channels)s/{}", base_pattern),
            "playlist_index" => format!("%(playlist_title|Playlists)s/{}", base_pattern),
            _ => base_pattern.to_string(),
        }
    }
}
