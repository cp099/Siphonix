use std::path::Path;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool, FromRow};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbJob {
    pub id: String,
    pub url: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub media_mode: String,
    pub format: String,
    pub quality: String,
    pub destination_path: String,
    pub state: String,
    pub progress: f64,
    pub download_speed: Option<String>,
    pub eta: Option<String>,
    pub file_size: Option<String>,
    pub error_message: Option<String>,
    pub last_error_category: Option<String>,
    pub retry_count: i64,
    pub max_retries: i64,
    pub next_retry_at: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

pub struct DbRepository {
    pool: SqlitePool,
    last_progress_write: Mutex<HashMap<String, Instant>>,
}

impl DbRepository {
    pub async fn init(db_path: &Path) -> Result<Self, sqlx::Error> {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());
        let opts = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(opts).await?;

        // Run embedded migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Self {
            pool,
            last_progress_write: Mutex::new(HashMap::new()),
        })
    }

    pub async fn insert_job(&self, job: &DbJob) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO jobs (
                id, url, title, thumbnail_url, media_mode, format, quality, destination_path,
                state, progress, download_speed, eta, file_size, error_message, last_error_category,
                retry_count, max_retries, next_retry_at, created_at, started_at, completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&job.id)
        .bind(&job.url)
        .bind(&job.title)
        .bind(&job.thumbnail_url)
        .bind(&job.media_mode)
        .bind(&job.format)
        .bind(&job.quality)
        .bind(&job.destination_path)
        .bind(&job.state)
        .bind(job.progress)
        .bind(&job.download_speed)
        .bind(&job.eta)
        .bind(&job.file_size)
        .bind(&job.error_message)
        .bind(&job.last_error_category)
        .bind(job.retry_count)
        .bind(job.max_retries)
        .bind(&job.next_retry_at)
        .bind(&job.created_at)
        .bind(&job.started_at)
        .bind(&job.completed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_job_state(
        &self,
        job_id: &str,
        state: &str,
        error_message: Option<&str>,
        last_error_category: Option<&str>,
        retry_count: i64,
        next_retry_at: Option<&str>,
        completed_at: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET state = ?, error_message = ?, last_error_category = ?, retry_count = ?, next_retry_at = ?, completed_at = ?
            WHERE id = ?
            "#
        )
        .bind(state)
        .bind(error_message)
        .bind(last_error_category)
        .bind(retry_count)
        .bind(next_retry_at)
        .bind(completed_at)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_job_progress_throttled(
        &self,
        job_id: &str,
        progress: f64,
        speed: Option<&str>,
        eta: Option<&str>,
        file_size: Option<&str>,
        force_now: bool,
    ) -> Result<bool, sqlx::Error> {
        let should_write = {
            let mut map = self.last_progress_write.lock().unwrap();
            let now = Instant::now();
            if force_now || map.get(job_id).map_or(true, |last| now.duration_since(*last) >= Duration::from_millis(800)) {
                map.insert(job_id.to_string(), now);
                true
            } else {
                false
            }
        };

        if should_write {
            sqlx::query(
                r#"
                UPDATE jobs
                SET progress = ?, download_speed = ?, eta = ?, file_size = ?
                WHERE id = ?
                "#
            )
            .bind(progress)
            .bind(speed)
            .bind(eta)
            .bind(file_size)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_all_jobs(&self) -> Result<Vec<DbJob>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DbJob>(
            r#"
            SELECT id, url, title, thumbnail_url, media_mode, format, quality, destination_path,
                   state, progress, download_speed, eta, file_size, error_message, last_error_category,
                   retry_count, max_retries, next_retry_at, created_at, started_at, completed_at
            FROM jobs
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_library_jobs(&self) -> Result<Vec<DbJob>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DbJob>(
            r#"
            SELECT id, url, title, thumbnail_url, media_mode, format, quality, destination_path,
                   state, progress, download_speed, eta, file_size, error_message, last_error_category,
                   retry_count, max_retries, next_retry_at, created_at, started_at, completed_at
            FROM jobs
            WHERE state = 'COMPLETED'
            ORDER BY completed_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn recover_interrupted_jobs(&self) -> Result<usize, sqlx::Error> {
        let res = sqlx::query(
            r#"
            UPDATE jobs
            SET state = 'QUEUED', progress = 0.0, download_speed = NULL, eta = NULL
            WHERE state IN ('DOWNLOADING', 'PREPARING', 'PROCESSING')
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected() as usize)
    }
}
