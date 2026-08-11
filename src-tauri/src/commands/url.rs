use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UrlType {
    VIDEO,
    PLAYLIST,
    VIDEO_WITH_PLAYLIST,
    INVALID,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlValidationResult {
    pub valid: bool,
    pub url_type: UrlType,
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
            url_type: UrlType::INVALID,
            is_playlist: false,
            video_id: None,
            playlist_id: None,
            message: "URL cannot be empty".into(),
        };
    }

    let playlist_param_regex = Regex::new(r"[?&]list=([a-zA-Z0-9_-]+)").unwrap();
    let video_param_regex = Regex::new(r"(?:v=|shorts/|youtu\.be/)([a-zA-Z0-9_-]{11})").unwrap();

    let found_playlist_id = playlist_param_regex.captures(trimmed).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
    let found_video_id = video_param_regex.captures(trimmed).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

    if found_video_id.is_some() && found_playlist_id.is_some() {
        return UrlValidationResult {
            valid: true,
            url_type: UrlType::VIDEO_WITH_PLAYLIST,
            is_playlist: true,
            video_id: found_video_id,
            playlist_id: found_playlist_id,
            message: "YouTube Video with Playlist parameter detected".into(),
        };
    }

    if found_playlist_id.is_some() && (trimmed.contains("youtube.com/playlist") || trimmed.contains("list=")) {
        return UrlValidationResult {
            valid: true,
            url_type: UrlType::PLAYLIST,
            is_playlist: true,
            video_id: None,
            playlist_id: found_playlist_id,
            message: "Valid YouTube Playlist detected".into(),
        };
    }

    if let Some(v_id) = found_video_id {
        return UrlValidationResult {
            valid: true,
            url_type: UrlType::VIDEO,
            is_playlist: false,
            video_id: Some(v_id),
            playlist_id: None,
            message: "Valid YouTube Video detected".into(),
        };
    }

    UrlValidationResult {
        valid: false,
        url_type: UrlType::INVALID,
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
        assert_eq!(res.url_type, UrlType::VIDEO);
        assert_eq!(res.video_id, Some("dQw4w9WgXcQ".to_string()));
    }

    #[test]
    fn test_valid_youtube_playlist_url() {
        let res = validate_url("https://www.youtube.com/playlist?list=PL3rVcngGfeeqE5H9N9".to_string());
        assert!(res.valid);
        assert_eq!(res.url_type, UrlType::PLAYLIST);
        assert_eq!(res.playlist_id, Some("PL3rVcngGfeeqE5H9N9".to_string()));
    }

    #[test]
    fn test_video_with_playlist_param_url() {
        let res = validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PL3rVcngGfeeqE5H9N9".to_string());
        assert!(res.valid);
        assert_eq!(res.url_type, UrlType::VIDEO_WITH_PLAYLIST);
        assert_eq!(res.video_id, Some("dQw4w9WgXcQ".to_string()));
        assert_eq!(res.playlist_id, Some("PL3rVcngGfeeqE5H9N9".to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let res = validate_url("https://example.com".to_string());
        assert!(!res.valid);
        assert_eq!(res.url_type, UrlType::INVALID);
    }
}
