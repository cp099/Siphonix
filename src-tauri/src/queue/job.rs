use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::state::JobState;
use crate::db::DbJob;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadJob {
    pub id: String,
    pub url: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub media_mode: String,
    pub format: String,
    pub quality: String,
    pub destination_path: String,
    pub state: JobState,
    pub progress: f64,
    pub download_speed: Option<String>,
    pub eta: Option<String>,
    pub file_size: Option<String>,
    pub error_message: Option<String>,
    pub last_error_category: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl DownloadJob {
    pub fn to_db_job(&self) -> DbJob {
        DbJob {
            id: self.id.clone(),
            url: self.url.clone(),
            title: self.title.clone(),
            thumbnail_url: self.thumbnail_url.clone(),
            media_mode: self.media_mode.clone(),
            format: self.format.clone(),
            quality: self.quality.clone(),
            destination_path: self.destination_path.clone(),
            state: self.state.as_str().to_string(),
            progress: self.progress,
            download_speed: self.download_speed.clone(),
            eta: self.eta.clone(),
            file_size: self.file_size.clone(),
            error_message: self.error_message.clone(),
            last_error_category: self.last_error_category.clone(),
            retry_count: self.retry_count as i64,
            max_retries: self.max_retries as i64,
            next_retry_at: self.next_retry_at.map(|dt| dt.to_rfc3339()),
            created_at: self.created_at.to_rfc3339(),
            started_at: self.started_at.map(|dt| dt.to_rfc3339()),
            completed_at: self.completed_at.map(|dt| dt.to_rfc3339()),
        }
    }

    pub fn from_db_job(db: DbJob) -> Self {
        Self {
            id: db.id,
            url: db.url,
            title: db.title,
            thumbnail_url: db.thumbnail_url,
            media_mode: db.media_mode,
            format: db.format,
            quality: db.quality,
            destination_path: db.destination_path,
            state: JobState::parse(&db.state),
            progress: db.progress,
            download_speed: db.download_speed,
            eta: db.eta,
            file_size: db.file_size,
            error_message: db.error_message,
            last_error_category: db.last_error_category,
            retry_count: db.retry_count as u32,
            max_retries: db.max_retries as u32,
            next_retry_at: db.next_retry_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            created_at: DateTime::parse_from_rfc3339(&db.created_at).map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
            started_at: db.started_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            completed_at: db.completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        }
    }
}
