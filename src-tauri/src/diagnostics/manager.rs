use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::event::{DiagnosticEvent, DiagnosticSeverity};
use super::health::{SystemHealth, SystemHealthEvaluator};
use super::logger::DiagnosticLogger;
use super::report::{DiagnosticReport, DiagnosticReportGenerator};
use super::storage::DiagnosticStorage;
use crate::db::DbRepository;
use crate::runtime::EngineManager;

#[derive(Clone)]
pub struct DiagnosticsManager {
    logger: DiagnosticLogger,
    storage: Arc<DiagnosticStorage>,
    engine_manager: Arc<EngineManager>,
}

impl DiagnosticsManager {
    pub fn new(app_data_dir: &Path, engine_manager: Arc<EngineManager>) -> Self {
        let logger = DiagnosticLogger::new(app_data_dir);
        let storage = Arc::new(DiagnosticStorage::new());

        let mgr = Self {
            logger,
            storage,
            engine_manager,
        };

        // Record initial system diagnostic event
        mgr.record_event(
            DiagnosticEvent::new(
                DiagnosticSeverity::Info,
                "system",
                "STARTUP",
                "Siphonix diagnostics subsystem initialized successfully.",
            )
        );

        mgr
    }

    pub fn record_event(&self, event: DiagnosticEvent) {
        // 1. Write to in-memory ring buffer
        self.storage.push(event.clone());
        // 2. Write to asynchronous non-blocking persistent file logger
        self.logger.log(event);
    }

    pub async fn get_system_health(&self, db: Option<&DbRepository>) -> SystemHealth {
        let runtime_status = self.engine_manager.get_status().await;

        let mut db_healthy = true;
        let mut failed_jobs_count = 0;
        let mut missing_library_files_count = 0;

        if let Some(db_repo) = db {
            if let Ok(jobs) = db_repo.get_all_jobs().await {
                failed_jobs_count = jobs.iter().filter(|j| j.state == "FAILED").count() as u32;
            } else {
                db_healthy = false;
            }

            if let Ok(lib_items) = db_repo.get_library_items(None, None, None, None).await {
                missing_library_files_count = lib_items.iter().filter(|i| i.file_status == "MISSING").count() as u32;
            }
        }

        SystemHealthEvaluator::evaluate(
            &runtime_status,
            db_healthy,
            failed_jobs_count,
            missing_library_files_count,
        )
    }

    pub async fn generate_report(&self, db: Option<&DbRepository>) -> DiagnosticReport {
        let health = self.get_system_health(db).await;
        let runtime_status = self.engine_manager.get_status().await;
        let recent_events = self.storage.get_recent(50);

        let mut jobs_count = 0;
        let mut lib_count = 0;

        if let Some(db_repo) = db {
            if let Ok(jobs) = db_repo.get_all_jobs().await {
                jobs_count = jobs.len() as u32;
            }
            if let Ok(items) = db_repo.get_library_items(None, None, None, None).await {
                lib_count = items.len() as u32;
            }
        }

        DiagnosticReportGenerator::generate(
            health,
            runtime_status,
            jobs_count,
            lib_count,
            recent_events,
        )
    }

    pub fn get_recent_events(&self, limit: usize) -> Vec<DiagnosticEvent> {
        self.storage.get_recent(limit)
    }
}
