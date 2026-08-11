use std::path::Path;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool, FromRow};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};
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
    pub source_video_id: Option<String>,
    pub source_playlist_id: Option<String>,
    pub source_playlist_title: Option<String>,
    pub playlist_entry_index: Option<i64>,
    pub options_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbLibraryItem {
    pub id: String,
    pub job_id: String,
    pub source_video_id: Option<String>,
    pub title: String,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub media_mode: String,
    pub format: String,
    pub quality: String,
    pub file_size_bytes: i64,
    pub duration_seconds: Option<i64>,
    pub thumbnail_url: Option<String>,
    pub source_url: String,
    pub source_playlist_id: Option<String>,
    pub source_playlist_title: Option<String>,
    pub playlist_entry_index: Option<i64>,
    pub created_at: String,
    pub completed_at: String,
    pub last_verified_at: String,
    pub file_status: String,
    pub options_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbPreset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: i64,
    pub options_json: String,
    pub created_at: String,
    pub updated_at: String,
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
                retry_count, max_retries, next_retry_at, created_at, started_at, completed_at,
                source_video_id, source_playlist_id, source_playlist_title, playlist_entry_index, options_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(&job.source_video_id)
        .bind(&job.source_playlist_id)
        .bind(&job.source_playlist_title)
        .bind(job.playlist_entry_index)
        .bind(&job.options_json)
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
                   retry_count, max_retries, next_retry_at, created_at, started_at, completed_at,
                   source_video_id, source_playlist_id, source_playlist_title, playlist_entry_index, options_json
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
                   retry_count, max_retries, next_retry_at, created_at, started_at, completed_at,
                   source_video_id, source_playlist_id, source_playlist_title, playlist_entry_index, options_json
            FROM jobs
            WHERE state = 'COMPLETED'
            ORDER BY completed_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Check if video IDs already exist in active/queued/completed states (excluding FAILED & CANCELLED)
    pub async fn find_existing_video_ids(&self, video_ids: &[String]) -> Result<HashSet<String>, sqlx::Error> {
        if video_ids.is_empty() {
            return Ok(HashSet::new());
        }

        let jobs = self.get_all_jobs().await?;
        let active_states: HashSet<&str> = [
            "QUEUED", "PREPARING", "DOWNLOADING", "PROCESSING", "RETRYING", "COOLDOWN", "COMPLETED"
        ].into_iter().collect();

        let mut existing = HashSet::new();
        for job in jobs {
            if active_states.contains(job.state.as_str()) {
                if let Some(v_id) = &job.source_video_id {
                    if video_ids.contains(v_id) {
                        existing.insert(v_id.clone());
                    }
                }
            }
        }

        Ok(existing)
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

    /// Insert a new LibraryItem record into library_items table (or update if file_path exists)
    pub async fn insert_library_item(&self, item: &DbLibraryItem) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO library_items (
                id, job_id, source_video_id, title, file_path, file_name, file_extension,
                media_mode, format, quality, file_size_bytes, duration_seconds, thumbnail_url,
                source_url, source_playlist_id, source_playlist_title, playlist_entry_index,
                created_at, completed_at, last_verified_at, file_status, options_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(file_path) DO UPDATE SET
                title = excluded.title,
                file_size_bytes = excluded.file_size_bytes,
                last_verified_at = excluded.last_verified_at,
                file_status = excluded.file_status,
                options_json = excluded.options_json
            "#
        )
        .bind(&item.id)
        .bind(&item.job_id)
        .bind(&item.source_video_id)
        .bind(&item.title)
        .bind(&item.file_path)
        .bind(&item.file_name)
        .bind(&item.file_extension)
        .bind(&item.media_mode)
        .bind(&item.format)
        .bind(&item.quality)
        .bind(item.file_size_bytes)
        .bind(item.duration_seconds)
        .bind(&item.thumbnail_url)
        .bind(&item.source_url)
        .bind(&item.source_playlist_id)
        .bind(&item.source_playlist_title)
        .bind(item.playlist_entry_index)
        .bind(&item.created_at)
        .bind(&item.completed_at)
        .bind(&item.last_verified_at)
        .bind(&item.file_status)
        .bind(&item.options_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Query library items with optional search, filter, and sort
    pub async fn get_library_items(
        &self,
        search: Option<&str>,
        filter_mode: Option<&str>,
        filter_status: Option<&str>,
        sort_by: Option<&str>,
    ) -> Result<Vec<DbLibraryItem>, sqlx::Error> {
        let mut query_str = String::from(
            "SELECT id, job_id, source_video_id, title, file_path, file_name, file_extension, \
             media_mode, format, quality, file_size_bytes, duration_seconds, thumbnail_url, \
             source_url, source_playlist_id, source_playlist_title, playlist_entry_index, \
             created_at, completed_at, last_verified_at, file_status, options_json FROM library_items WHERE 1=1",
        );

        if let Some(s) = search {
            if !s.trim().is_empty() {
                query_str.push_str(" AND (title LIKE '%");
                query_str.push_str(&s.replace('\'', "''"));
                query_str.push_str("%' OR file_name LIKE '%");
                query_str.push_str(&s.replace('\'', "''"));
                query_str.push_str("%' OR source_video_id LIKE '%");
                query_str.push_str(&s.replace('\'', "''"));
                query_str.push_str("%' OR source_playlist_title LIKE '%");
                query_str.push_str(&s.replace('\'', "''"));
                query_str.push_str("%')");
            }
        }

        if let Some(m) = filter_mode {
            if m != "ALL" && !m.trim().is_empty() {
                query_str.push_str(" AND media_mode = '");
                query_str.push_str(&m.replace('\'', "''"));
                query_str.push_str("'");
            }
        }

        if let Some(st) = filter_status {
            if st != "ALL" && !st.trim().is_empty() {
                query_str.push_str(" AND file_status = '");
                query_str.push_str(&st.replace('\'', "''"));
                query_str.push_str("'");
            }
        }

        match sort_by {
            Some("oldest") => query_str.push_str(" ORDER BY completed_at ASC"),
            Some("title_asc") => query_str.push_str(" ORDER BY title ASC"),
            Some("title_desc") => query_str.push_str(" ORDER BY title DESC"),
            Some("size_desc") => query_str.push_str(" ORDER BY file_size_bytes DESC"),
            Some("duration_desc") => query_str.push_str(" ORDER BY duration_seconds DESC"),
            _ => query_str.push_str(" ORDER BY completed_at DESC"),
        }

        let rows = sqlx::query_as::<_, DbLibraryItem>(&query_str)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn get_library_item_by_id(&self, id: &str) -> Result<Option<DbLibraryItem>, sqlx::Error> {
        let row = sqlx::query_as::<_, DbLibraryItem>(
            "SELECT id, job_id, source_video_id, title, file_path, file_name, file_extension, \
             media_mode, format, quality, file_size_bytes, duration_seconds, thumbnail_url, \
             source_url, source_playlist_id, source_playlist_title, playlist_entry_index, \
             created_at, completed_at, last_verified_at, file_status, options_json FROM library_items WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_library_item_status(&self, id: &str, status: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE library_items SET file_status = ?, last_verified_at = ? WHERE id = ?"
        )
        .bind(status)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_library_item(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM library_items WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- Preset Operations ---

    pub async fn insert_preset(&self, preset: &DbPreset) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO presets (id, name, description, is_default, options_json, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                is_default = excluded.is_default,
                options_json = excluded.options_json,
                updated_at = excluded.updated_at
            "#
        )
        .bind(&preset.id)
        .bind(&preset.name)
        .bind(&preset.description)
        .bind(preset.is_default)
        .bind(&preset.options_json)
        .bind(&preset.created_at)
        .bind(&preset.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_presets(&self) -> Result<Vec<DbPreset>, sqlx::Error> {
        sqlx::query_as::<_, DbPreset>(
            "SELECT id, name, description, is_default, options_json, created_at, updated_at FROM presets ORDER BY is_default DESC, name ASC"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_preset_by_id(&self, id: &str) -> Result<Option<DbPreset>, sqlx::Error> {
        sqlx::query_as::<_, DbPreset>(
            "SELECT id, name, description, is_default, options_json, created_at, updated_at FROM presets WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_preset(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM presets WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Enforce single default preset using an atomic SQLite transaction
    pub async fn set_default_preset(&self, id: &str) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("UPDATE presets SET is_default = 0")
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE presets SET is_default = 1 WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
