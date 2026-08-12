use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::db::DbRepository;
use crate::library::LibraryService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub subsystem: String,
    pub success: bool,
    pub message: String,
    pub items_affected: u32,
}

pub struct RecoveryManager;

impl RecoveryManager {
    pub async fn verify_database(db: &DbRepository) -> RecoveryResult {
        match db.get_all_jobs().await {
            Ok(jobs) => RecoveryResult {
                subsystem: "database".to_string(),
                success: true,
                message: format!("SQLite database query verified. Total jobs recorded: {}", jobs.len()),
                items_affected: jobs.len() as u32,
            },
            Err(e) => RecoveryResult {
                subsystem: "database".to_string(),
                success: false,
                message: format!("Database verification failed: {}", e),
                items_affected: 0,
            },
        }
    }

    pub async fn recover_interrupted_jobs(db: &DbRepository) -> RecoveryResult {
        match db.get_all_jobs().await {
            Ok(jobs) => {
                let mut recovered = 0;
                for job in jobs {
                    if job.state == "DOWNLOADING" || job.state == "PREPARING" || job.state == "PROCESSING" {
                        let _ = db.update_job_state(&job.id, "QUEUED", None, None, job.retry_count, None, None).await;
                        recovered += 1;
                    }
                }
                RecoveryResult {
                    subsystem: "queue".to_string(),
                    success: true,
                    message: format!("Recovered {} interrupted job(s) back to QUEUED state.", recovered),
                    items_affected: recovered,
                }
            }
            Err(e) => RecoveryResult {
                subsystem: "queue".to_string(),
                success: false,
                message: format!("Interrupted job recovery failed: {}", e),
                items_affected: 0,
            },
        }
    }

    pub async fn verify_library(db: Arc<DbRepository>) -> RecoveryResult {
        let lib_service = LibraryService::new(db);
        match lib_service.verify_library_items().await {
            Ok(updated) => RecoveryResult {
                subsystem: "library".to_string(),
                success: true,
                message: format!("Library file status verification complete. Verified {} item(s).", updated.len()),
                items_affected: updated.len() as u32,
            },
            Err(e) => RecoveryResult {
                subsystem: "library".to_string(),
                success: false,
                message: format!("Library verification failed: {}", e),
                items_affected: 0,
            },
        }
    }
}
