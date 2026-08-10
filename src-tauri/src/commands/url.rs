use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlValidationResult {
    pub valid: bool,
    pub is_playlist: bool,
    pub video_id: Option<String>,
    pub playlist_id: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn validate_url(url: String) -> UrlValidationResult {
    let trimmed = url.trim();

    if trimmed.is_empty() {
        return UrlValidationResult {
            valid: false,
            is_playlist: false,
            video_id: None,
            playlist_id: None,
            message: "URL cannot be empty".into(),
        };
    }

    // Check for YouTube playlist
    let playlist_regex = Regex::new(r"(?:youtube\.com/playlist\?list=)([a-zA-Z0-9_-]+)").unwrap();
    if let Some(captures) = playlist_regex.captures(trimmed) {
        if let Some(list_id) = captures.get(1) {
            return UrlValidationResult {
                valid: true,
                is_playlist: true,
                video_id: None,
                playlist_id: Some(list_id.as_str().to_string()),
                message: "Valid YouTube Playlist detected".into(),
            };
        }
    }

    // Check for YouTube video (watch?v=, Shorts, or youtu.be)
    let video_regex = Regex::new(
        r"(?:youtube\.com/(?:watch\?v=|shorts/)|youtu\.be/)([a-zA-Z0-9_-]{11})"
    ).unwrap();

    if let Some(captures) = video_regex.captures(trimmed) {
        if let Some(v_id) = captures.get(1) {
            return UrlValidationResult {
                valid: true,
                is_playlist: false,
                video_id: Some(v_id.as_str().to_string()),
                playlist_id: None,
                message: "Valid YouTube Video detected".into(),
            };
        }
    }

    UrlValidationResult {
        valid: false,
        is_playlist: false,
        video_id: None,
        playlist_id: None,
        message: "Please enter a valid YouTube video or playlist URL".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_youtube_video_url() {
        let res = validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());
        assert!(res.valid);
        assert!(!res.is_playlist);
        assert_eq!(res.video_id, Some("dQw4w9WgXcQ".to_string()));
    }

    #[test]
    fn test_valid_youtube_playlist_url() {
        let res = validate_url("https://www.youtube.com/playlist?list=PL3rVcngGfeeqE5H9N9".to_string());
        assert!(res.valid);
        assert!(res.is_playlist);
        assert_eq!(res.playlist_id, Some("PL3rVcngGfeeqE5H9N9".to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let res = validate_url("https://example.com".to_string());
        assert!(!res.valid);
    }
}

