use regex::Regex;
use serde::{Deserialize, Serialize};
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
    pub media_mode: String,
    pub video: VideoOptions,
    pub audio: AudioOptions,
    pub output: OutputOptions,
    pub metadata: MetadataOptions,
    pub subtitles: SubtitleOptions,
    pub network: NetworkOptions,
    pub expert: ExpertOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VideoOptions {
    pub resolution: String,
    pub frame_rate: String,
    pub codec_preference: String,
    pub hdr_preference: String,
    pub selection_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AudioOptions {
    pub format: String,
    pub quality: String,
    pub codec_preference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct OutputOptions {
    pub container: String,
    pub destination_path: String,
    pub naming_preset: String,
    pub custom_naming_template: Option<String>,
    pub folder_organization: String,
    pub overwrite_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MetadataOptions {
    pub embed_metadata: bool,
    pub embed_thumbnail: bool,
    pub write_metadata_json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SubtitleOptions {
    pub enabled: bool,
    pub languages: Vec<String>,
    pub format: String,
    pub embed_in_video: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NetworkOptions {
    pub concurrent_fragments: String,
    pub max_download_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ExpertOptions {
    pub format_sort_strategy: String,
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
        if self.media_mode == "audio" {
            self.subtitles.enabled = false;
            self.subtitles.embed_in_video = false;
        }

        if self.media_mode == "audio" {
            let fmt = self.audio.format.to_uppercase();
            if fmt == "FLAC" || fmt == "WAV" || fmt == "ALAC" {
                self.audio.quality = "best".to_string();
            }
        }

        if self.output.naming_preset == "custom" {
            if let Some(ref template) = self.output.custom_naming_template {
                Self::validate_template_string(template)?;
            } else {
                return Err(ValidationError::InvalidNamingTemplate(
                    "Custom naming preset selected but no template provided".to_string(),
                ));
            }
        }

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

        if !self.output.destination_path.is_empty() {
            let dest_trimmed = self.output.destination_path.trim();
            if dest_trimmed == "/"
                || dest_trimmed == "/etc"
                || dest_trimmed == "/bin"
                || dest_trimmed == "/usr"
                || dest_trimmed == "/System"
                || dest_trimmed == "C:\\"
                || dest_trimmed == "C:\\Windows"
                || dest_trimmed == "C:\\Windows\\System32"
            {
                return Err(ValidationError::InvalidPath(
                    "Selected destination path is a protected system directory".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn validate_template_string(template: &str) -> Result<(), ValidationError> {
        let t = template.trim();
        if t.is_empty() {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template cannot be empty".to_string(),
            ));
        }

        if t.contains('/') || t.contains('\\') || t.contains("..") || t.starts_with('/') {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template cannot contain path separators or relative directory traversal"
                    .to_string(),
            ));
        }

        let forbidden_shell_chars = ['$', '|', ';', '&', '`', '>', '<'];
        if t.chars().any(|c| forbidden_shell_chars.contains(&c)) {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template contains forbidden shell control characters".to_string(),
            ));
        }

        let win_reserved = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        if t.chars().any(|c| win_reserved.contains(&c)) {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template contains Windows reserved characters".to_string(),
            ));
        }

        let re = get_template_regex();
        if !re.is_match(t) {
            return Err(ValidationError::InvalidNamingTemplate(
                "Template contains unsupported yt-dlp field syntax or format specifiers"
                    .to_string(),
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
            _ => "%(title)s.%(ext)s",
        };

        match self.output.folder_organization.as_str() {
            "by_playlist" => format!("%(playlist_title|Playlists)s/{}", base_pattern),
            "by_channel" => format!("%(uploader|Channels)s/{}", base_pattern),
            "playlist_index" => format!("%(playlist_title|Playlists)s/{}", base_pattern),
            _ => base_pattern.to_string(),
        }
    }
}
