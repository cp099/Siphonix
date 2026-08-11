use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVersion {
    pub raw: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    YtDlp,
    Ffmpeg,
}

impl EngineType {
    pub fn name(&self) -> &'static str {
        match self {
            EngineType::YtDlp => "yt-dlp",
            EngineType::Ffmpeg => "ffmpeg",
        }
    }
}

pub struct VersionChecker;

impl VersionChecker {
    pub async fn detect_and_verify(
        path: &Path,
        engine_type: EngineType,
    ) -> Result<ParsedVersion, String> {
        if !path.exists() || !path.is_file() {
            return Err(format!("Binary does not exist at {}", path.display()));
        }

        // Direct process execution strictly without shell wrappers (no sh -c / cmd /c)
        let mut cmd = Command::new(path);
        match engine_type {
            EngineType::YtDlp => {
                cmd.arg("--version");
            }
            EngineType::Ffmpeg => {
                cmd.arg("-version");
            }
        }

        let output = cmd.output().await.map_err(|e| format!("Failed to execute process: {}", e))?;

        if !output.status.success() {
            return Err(format!("Process exited with non-zero status code: {:?}", output.status.code()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{}\n{}", stdout, stderr);

        match engine_type {
            EngineType::YtDlp => Self::parse_ytdlp_version(&stdout, &combined),
            EngineType::Ffmpeg => Self::parse_ffmpeg_version(&stdout, &combined),
        }
    }

    pub fn parse_ytdlp_version(stdout: &str, full_output: &str) -> Result<ParsedVersion, String> {
        let first_line = stdout.lines().next().unwrap_or("").trim();
        if first_line.is_empty() {
            return Err("yt-dlp output empty".to_string());
        }

        // Validate identity: output must be date-like (YYYY.MM.DD) or contain yt-dlp
        let parts: Vec<&str> = first_line.split('.').collect();
        if parts.len() >= 3 {
            let year = parts[0].parse::<u32>().map_err(|_| "Invalid yt-dlp year integer".to_string())?;
            let month = parts[1].parse::<u32>().map_err(|_| "Invalid yt-dlp month integer".to_string())?;
            let day = parts[2].parse::<u32>().map_err(|_| "Invalid yt-dlp day integer".to_string())?;

            if year < 2020 {
                return Err(format!("yt-dlp version year {} is outdated or invalid", year));
            }

            Ok(ParsedVersion {
                raw: first_line.to_string(),
                major: year,
                minor: month,
                patch: day,
            })
        } else if full_output.to_lowercase().contains("yt-dlp") {
            Ok(ParsedVersion {
                raw: first_line.to_string(),
                major: 2024,
                minor: 1,
                patch: 1,
            })
        } else {
            Err(format!("Identity validation failed: Output line '{}' does not match yt-dlp version format", first_line))
        }
    }

    pub fn parse_ffmpeg_version(stdout: &str, full_output: &str) -> Result<ParsedVersion, String> {
        let lower = full_output.to_lowercase();
        if !lower.contains("ffmpeg") {
            return Err("Identity validation failed: Executable output does not contain 'ffmpeg'".to_string());
        }

        let first_line = stdout.lines().next().unwrap_or("").trim();
        if let Some(pos) = lower.find("ffmpeg version ") {
            let rest = &stdout[pos + "ffmpeg version ".len()..];
            let ver_token = rest.split_whitespace().next().unwrap_or("");
            let clean_ver = ver_token.trim_start_matches('n');
            let parts: Vec<&str> = clean_ver.split('.').collect();

            if parts.len() >= 2 {
                let major = parts[0].parse::<u32>().unwrap_or(4);
                let minor = parts[1].parse::<u32>().unwrap_or(0);
                let patch = if parts.len() > 2 { parts[2].parse::<u32>().unwrap_or(0) } else { 0 };

                return Ok(ParsedVersion {
                    raw: clean_ver.to_string(),
                    major,
                    minor,
                    patch,
                });
            }
        }

        // Generic fallback for git/snapshot builds of ffmpeg
        Ok(ParsedVersion {
            raw: if first_line.is_empty() { "ffmpeg-snapshot".to_string() } else { first_line.to_string() },
            major: 4,
            minor: 0,
            patch: 0,
        })
    }

    pub fn is_compatible(version: &ParsedVersion, engine_type: EngineType) -> bool {
        match engine_type {
            EngineType::YtDlp => version.major >= 2022,
            EngineType::Ffmpeg => version.major >= 4,
        }
    }
}
