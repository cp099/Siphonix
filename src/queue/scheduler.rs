use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::backoff::BackoffPolicy;
use super::cooldown::{CooldownManager, CooldownStatus};
use super::job::DownloadJob;
use super::state::JobState;
use crate::db::DbRepository;
use crate::engine::builder::DownloadRequest;
use crate::engine::DownloadManager;
use crate::library::LibraryService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSummary {
    pub active_count: usize,
    pub waiting_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub max_concurrency: usize,
    pub is_paused: bool,
    pub cooldown: CooldownStatus,
}

pub struct QueueScheduler {
    db: Arc<DbRepository>,
    download_manager: DownloadManager,
    jobs: Mutex<Vec<DownloadJob>>,
    active_ids: Mutex<Vec<String>>,
    max_concurrency: Mutex<usize>,
    is_paused: Mutex<bool>,
    cooldown_mgr: Mutex<CooldownManager>,
    backoff_policy: BackoffPolicy,
    library_service: LibraryService,
}

impl QueueScheduler {
    pub async fn new(db: Arc<DbRepository>, download_manager: DownloadManager) -> Arc<Self> {
        let _ = db.recover_interrupted_jobs().await;

        let db_jobs = db.get_all_jobs().await.unwrap_or_default();
        let jobs = db_jobs.into_iter().map(DownloadJob::from_db_job).collect();
        let library_service = LibraryService::new(Arc::clone(&db));

        Arc::new(Self {
            db,
            download_manager,
            jobs: Mutex::new(jobs),
            active_ids: Mutex::new(Vec::new()),
            max_concurrency: Mutex::new(2),
            is_paused: Mutex::new(false),
            cooldown_mgr: Mutex::new(CooldownManager::default()),
            backoff_policy: BackoffPolicy::default(),
            library_service,
        })
    }

    pub fn start_worker_loop(self: &Arc<Self>) {
        let scheduler = Arc::clone(self);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            loop {
                interval.tick().await;
                scheduler.tick_scheduler_with_arc(&scheduler).await;
            }
        });
    }

    pub async fn tick_scheduler(&self) {
        self.tick_scheduler_internal(None).await;
    }

    pub async fn tick_scheduler_with_arc(&self, scheduler_arc: &Arc<Self>) {
        self.tick_scheduler_internal(Some(scheduler_arc.clone()))
            .await;
    }

    async fn tick_scheduler_internal(&self, scheduler_arc: Option<Arc<Self>>) {
        let is_paused = *self.is_paused.lock().await;
        let max_conc = *self.max_concurrency.lock().await;
        let cooldown = self.cooldown_mgr.lock().await;

        let cooldown_active = cooldown.is_cooldown_active();
        if is_paused || cooldown_active {
            return;
        }
        drop(cooldown);

        let mut jobs_guard = self.jobs.lock().await;
        let mut active_guard = self.active_ids.lock().await;

        let now = Utc::now();

        for job in jobs_guard.iter_mut() {
            if job.state == JobState::RETRYING {
                if let Some(retry_at) = job.next_retry_at {
                    if now >= retry_at {
                        job.state = JobState::QUEUED;
                        job.next_retry_at = None;
                        let _ = self
                            .db
                            .update_job_state(
                                &job.id,
                                job.state.as_str(),
                                job.error_message.as_deref(),
                                job.last_error_category.as_deref(),
                                job.retry_count as i64,
                                None,
                                None,
                            )
                            .await;
                    }
                }
            }
        }

        while active_guard.len() < max_conc {
            let eligible_pos = jobs_guard.iter().position(|j| j.state == JobState::QUEUED);

            if let Some(pos) = eligible_pos {
                let job = &mut jobs_guard[pos];
                job.state = JobState::PREPARING;
                job.started_at = Some(now);

                let job_id = job.id.clone();
                let req = DownloadRequest {
                    url: job.url.clone(),
                    media_mode: job.media_mode.clone(),
                    audio_format: Some(job.format.clone()),
                    audio_quality: Some(job.quality.clone()),
                    video_format: Some(job.format.clone()),
                    video_quality: Some(job.quality.clone()),
                    destination_path: job.destination_path.clone(),
                    options: Some(job.options.clone()),
                };

                let _ = self
                    .db
                    .update_job_state(
                        &job_id,
                        "PREPARING",
                        None,
                        None,
                        job.retry_count as i64,
                        None,
                        None,
                    )
                    .await;

                active_guard.push(job_id.clone());

                let mgr = self.download_manager.clone();
                let j_id = job_id.clone();
                let sched_opt = scheduler_arc.clone();

                tokio::spawn(async move {
                    if let Some(sched) = sched_opt {
                        let s_clone = sched.clone();
                        let _ = mgr
                            .start_download_with_callback(
                                j_id,
                                req,
                                Some(move |update: crate::engine::runner::ProgressUpdate| {
                                    let s = s_clone.clone();
                                    tokio::spawn(async move {
                                        s.handle_progress_event(
                                            &update.job_id,
                                            &update.state,
                                            update.progress as f64,
                                            update.speed.as_deref(),
                                            update.eta.as_deref(),
                                            update.file_size.as_deref(),
                                            update.error_message.as_deref(),
                                            update.final_output_path.as_deref(),
                                        )
                                        .await;
                                    });
                                }),
                            )
                            .await;
                    } else {
                        let _ = mgr.start_download(j_id, req).await;
                    }
                });
            } else {
                break;
            }
        }
    }

    pub async fn enqueue_job(&self, job: DownloadJob) -> Result<DownloadJob, String> {
        let db_job = job.to_db_job();
        self.db
            .insert_job(&db_job)
            .await
            .map_err(|e| e.to_string())?;

        let mut jobs = self.jobs.lock().await;
        jobs.push(job.clone());
        Ok(job)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn handle_progress_event(
        &self,
        job_id: &str,
        state_str: &str,
        progress: f64,
        speed: Option<&str>,
        eta: Option<&str>,
        file_size: Option<&str>,
        error_msg: Option<&str>,
        captured_filepath: Option<&str>,
    ) {
        let mut jobs = self.jobs.lock().await;
        let mut active = self.active_ids.lock().await;

        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            let new_state = JobState::parse(state_str);
            let state_changed = job.state != new_state;
            job.state = new_state;
            job.progress = progress;
            job.download_speed = speed.map(String::from);
            job.eta = eta.map(String::from);
            job.file_size = file_size.map(String::from);

            if let Some(err) = error_msg {
                job.error_message = Some(err.to_string());
            }

            if new_state == JobState::COMPLETED
                || new_state == JobState::FAILED
                || new_state == JobState::CANCELLED
                || new_state == JobState::RETRYING
            {
                active.retain(|id| id != job_id);
            }

            if new_state == JobState::COMPLETED {
                job.completed_at = Some(Utc::now());
                let _ = self
                    .library_service
                    .register_completed_job(job, captured_filepath)
                    .await;
            }

            if new_state == JobState::FAILED {
                DownloadManager::cleanup_partial_files(
                    Path::new(&job.destination_path),
                    &job.title,
                );
                if let Some(err) = error_msg {
                    if err.to_lowercase().contains("http error 429")
                        || err.to_lowercase().contains("rate limited")
                    {
                        let mut cooldown = self.cooldown_mgr.lock().await;
                        cooldown.record_rate_limit();
                    }

                    if self.backoff_policy.is_retryable(job.retry_count) {
                        job.retry_count += 1;
                        let delay = self.backoff_policy.calculate_delay(job.retry_count);
                        let next_retry =
                            Utc::now() + chrono::Duration::from_std(delay).unwrap_or_default();
                        job.next_retry_at = Some(next_retry);
                        job.state = JobState::RETRYING;

                        let _ = self
                            .db
                            .update_job_state(
                                job_id,
                                "RETRYING",
                                job.error_message.as_deref(),
                                Some("TemporaryNetworkError"),
                                job.retry_count as i64,
                                Some(&next_retry.to_rfc3339()),
                                None,
                            )
                            .await;
                        return;
                    }
                }
            }

            if state_changed || new_state == JobState::COMPLETED || new_state == JobState::FAILED {
                let _ = self
                    .db
                    .update_job_state(
                        job_id,
                        job.state.as_str(),
                        job.error_message.as_deref(),
                        job.last_error_category.as_deref(),
                        job.retry_count as i64,
                        job.next_retry_at.map(|d| d.to_rfc3339()).as_deref(),
                        job.completed_at.map(|d| d.to_rfc3339()).as_deref(),
                    )
                    .await;
            } else {
                let _ = self
                    .db
                    .update_job_progress_throttled(job_id, progress, speed, eta, file_size, false)
                    .await;
            }
        }
    }

    pub async fn set_pause_queue(&self, pause: bool) {
        let mut p = self.is_paused.lock().await;
        *p = pause;
    }

    pub async fn force_resume_cooldown(&self) {
        let mut cooldown = self.cooldown_mgr.lock().await;
        cooldown.force_resume();
    }

    pub async fn cancel_job(&self, job_id: &str) -> bool {
        let _ = self.download_manager.cancel_download(job_id).await;

        let mut jobs = self.jobs.lock().await;
        let mut active = self.active_ids.lock().await;

        active.retain(|id| id != job_id);

        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.state = JobState::CANCELLED;
            let _ = self
                .db
                .update_job_state(
                    job_id,
                    "CANCELLED",
                    None,
                    None,
                    job.retry_count as i64,
                    None,
                    None,
                )
                .await;
            true
        } else {
            false
        }
    }

    pub async fn set_max_concurrency(&self, limit: usize) {
        let mut c = self.max_concurrency.lock().await;
        *c = limit.clamp(1, 4);
    }

    pub async fn get_all_jobs(&self) -> Vec<DownloadJob> {
        self.jobs.lock().await.clone()
    }

    pub async fn get_library_jobs(&self) -> Vec<DownloadJob> {
        self.db
            .get_library_jobs()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(DownloadJob::from_db_job)
            .collect()
    }

    pub async fn get_summary(&self) -> QueueSummary {
        let jobs = self.jobs.lock().await;
        let active = self.active_ids.lock().await;
        let max_c = *self.max_concurrency.lock().await;
        let is_p = *self.is_paused.lock().await;
        let cooldown = self.cooldown_mgr.lock().await.get_status();

        QueueSummary {
            active_count: active.len(),
            waiting_count: jobs
                .iter()
                .filter(|j| j.state == JobState::QUEUED || j.state == JobState::RETRYING)
                .count(),
            completed_count: jobs
                .iter()
                .filter(|j| j.state == JobState::COMPLETED)
                .count(),
            failed_count: jobs.iter().filter(|j| j.state == JobState::FAILED).count(),
            max_concurrency: max_c,
            is_paused: is_p,
            cooldown,
        }
    }
}
