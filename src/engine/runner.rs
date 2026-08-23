use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub job_id: String,
    pub state: String,
    pub progress: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub file_size: Option<String>,
    pub error_message: Option<String>,
    pub final_output_path: Option<String>,
}

pub struct OutputParser;

impl OutputParser {
    pub fn is_filepath_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('[')
            || trimmed.starts_with("SIPHONIX_PROGRESS|")
            || trimmed.starts_with("WARNING:")
            || trimmed.starts_with("ERROR:")
        {
            return false;
        }
        std::path::Path::new(trimmed).is_absolute()
    }

    pub fn extract_filepath(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("[Merger] Merging formats into \"") && trimmed.ends_with('"') {
            let path = &trimmed[30..trimmed.len() - 1];
            return Some(path.to_string());
        }
        if trimmed.starts_with("[download] Destination: ") {
            let path = trimmed[23..].trim();
            if !path.ends_with(".part") && !path.ends_with(".ytdl") {
                return Some(path.to_string());
            }
        }
        if trimmed.starts_with("[ExtractAudio] Destination: ") {
            return Some(trimmed[27..].trim().to_string());
        }
        if Self::is_filepath_line(trimmed) {
            return Some(trimmed.to_string());
        }
        None
    }

    pub fn parse_line(job_id: &str, line: &str) -> Option<ProgressUpdate> {
        let trimmed = line.trim();

        if trimmed.starts_with("SIPHONIX_PROGRESS|") {
            let parts: Vec<&str> = trimmed.split('|').collect();
            if parts.len() >= 6 {
                let status = parts[1];
                let raw_percent = parts[2].trim().trim_end_matches('%');
                let percent = raw_percent.parse::<f32>().unwrap_or(0.0);
                let speed = if parts[3].trim().is_empty() || parts[3] == "NA" {
                    None
                } else {
                    Some(parts[3].trim().to_string())
                };
                let eta = if parts[4].trim().is_empty() || parts[4] == "NA" {
                    None
                } else {
                    Some(parts[4].trim().to_string())
                };
                let file_size = if parts[5].trim().is_empty() || parts[5] == "NA" {
                    None
                } else {
                    Some(parts[5].trim().to_string())
                };

                let state = if status.contains("finished") || percent >= 100.0 {
                    "PROCESSING".to_string()
                } else {
                    "DOWNLOADING".to_string()
                };

                return Some(ProgressUpdate {
                    job_id: job_id.to_string(),
                    state,
                    progress: percent,
                    speed,
                    eta,
                    file_size,
                    error_message: None,
                    final_output_path: None,
                });
            }
        }

        if trimmed.starts_with("[download]") && trimmed.contains('%') {
            if let Some(pct_idx) = trimmed.find('%') {
                let before_pct = &trimmed[..pct_idx];
                if let Some(last_space) = before_pct.rfind(|c: char| c.is_whitespace() || c == ']')
                {
                    let pct_str = before_pct[last_space + 1..].trim();
                    if let Ok(pct) = pct_str.parse::<f32>() {
                        let speed = if let Some(at_idx) = trimmed.find(" at ") {
                            let after_at = &trimmed[at_idx + 4..];
                            let end_speed = after_at
                                .find(|c: char| c.is_whitespace())
                                .unwrap_or(after_at.len());
                            let s = after_at[..end_speed].trim();
                            if !s.is_empty() && s != "Unknown" {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        let eta = if let Some(eta_idx) = trimmed.find("ETA ") {
                            let after_eta = &trimmed[eta_idx + 4..];
                            let end_eta = after_eta
                                .find(|c: char| c.is_whitespace())
                                .unwrap_or(after_eta.len());
                            let e = after_eta[..end_eta].trim();
                            if !e.is_empty() && e != "Unknown" {
                                Some(e.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        return Some(ProgressUpdate {
                            job_id: job_id.to_string(),
                            state: if pct >= 100.0 {
                                "PROCESSING".to_string()
                            } else {
                                "DOWNLOADING".to_string()
                            },
                            progress: pct,
                            speed,
                            eta,
                            file_size: None,
                            error_message: None,
                            final_output_path: None,
                        });
                    }
                }
            }
        }

        if trimmed.starts_with("[ExtractAudio]")
            || trimmed.starts_with("[Merger]")
            || trimmed.starts_with("[VideoConvertor]")
            || trimmed.starts_with("[ffmpeg]")
        {
            return Some(ProgressUpdate {
                job_id: job_id.to_string(),
                state: "PROCESSING".to_string(),
                progress: 99.0,
                speed: None,
                eta: None,
                file_size: None,
                error_message: None,
                final_output_path: None,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_structured_progress() {
        let line = "SIPHONIX_PROGRESS|downloading| 45.2%| 5.4MiB/s| 00:12| 34.5MiB";
        let update = OutputParser::parse_line("job-1", line).unwrap();
        assert_eq!(update.job_id, "job-1");
        assert_eq!(update.state, "DOWNLOADING");
        assert_eq!(update.progress, 45.2);
        assert_eq!(update.speed, Some("5.4MiB/s".to_string()));
        assert_eq!(update.eta, Some("00:12".to_string()));
    }

    #[test]
    fn test_parse_post_processing() {
        let line = "[ExtractAudio] Destination: /path/song.mp3";
        let update = OutputParser::parse_line("job-1", line).unwrap();
        assert_eq!(update.state, "PROCESSING");
    }
}
