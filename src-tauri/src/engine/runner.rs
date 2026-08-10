use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub job_id: String,
    pub state: String, // "PREPARING" | "DOWNLOADING" | "PROCESSING" | "COMPLETED" | "FAILED"
    pub progress: f32, // 0.0 to 100.0
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub file_size: Option<String>,
    pub error_message: Option<String>,
}

pub struct OutputParser;

impl OutputParser {
    pub fn parse_line(job_id: &str, line: &str) -> Option<ProgressUpdate> {
        let trimmed = line.trim();

        // 1. Structured Progress Template parsing: SIPHONIX_PROGRESS|status|percent|speed|eta|total_bytes
        if trimmed.starts_with("SIPHONIX_PROGRESS|") {
            let parts: Vec<&str> = trimmed.split('|').collect();
            if parts.len() >= 6 {
                let status = parts[1];
                let raw_percent = parts[2].trim().trim_end_matches('%');
                let percent = raw_percent.parse::<f32>().unwrap_or(0.0);
                let speed = if parts[3].trim().is_empty() || parts[3] == "NA" { None } else { Some(parts[3].trim().to_string()) };
                let eta = if parts[4].trim().is_empty() || parts[4] == "NA" { None } else { Some(parts[4].trim().to_string()) };
                let file_size = if parts[5].trim().is_empty() || parts[5] == "NA" { None } else { Some(parts[5].trim().to_string()) };

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
                });
            }
        }

        // 2. Post-processing state detection
        if trimmed.starts_with("[ExtractAudio]") || trimmed.starts_with("[Merger]") || trimmed.starts_with("[VideoConvertor]") || trimmed.starts_with("[ffmpeg]") {
            return Some(ProgressUpdate {
                job_id: job_id.to_string(),
                state: "PROCESSING".to_string(),
                progress: 98.0,
                speed: None,
                eta: None,
                file_size: None,
                error_message: None,
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
