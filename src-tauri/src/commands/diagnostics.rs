use std::sync::Arc;
use tauri::State;

use crate::db::DbRepository;
use crate::diagnostics::{
    DiagnosticEvent, DiagnosticReport, DiagnosticsManager, RecoveryManager, RecoveryResult, SystemHealth,
};

#[tauri::command]
pub async fn get_diagnostics(
    diag_mgr: State<'_, Arc<DiagnosticsManager>>,
    limit: Option<usize>,
) -> Result<Vec<DiagnosticEvent>, String> {
    Ok(diag_mgr.get_recent_events(limit.unwrap_or(50)))
}

#[tauri::command]
pub async fn get_system_health(
    diag_mgr: State<'_, Arc<DiagnosticsManager>>,
    db: State<'_, Arc<DbRepository>>,
) -> Result<SystemHealth, String> {
    Ok(diag_mgr.get_system_health(Some(&db)).await)
}

#[tauri::command]
pub async fn generate_diagnostic_report(
    diag_mgr: State<'_, Arc<DiagnosticsManager>>,
    db: State<'_, Arc<DbRepository>>,
) -> Result<DiagnosticReport, String> {
    Ok(diag_mgr.generate_report(Some(&db)).await)
}

#[tauri::command]
pub async fn verify_database(
    db: State<'_, Arc<DbRepository>>,
) -> Result<RecoveryResult, String> {
    Ok(RecoveryManager::verify_database(&db).await)
}

#[tauri::command]
pub async fn verify_library(
    db: State<'_, Arc<DbRepository>>,
) -> Result<RecoveryResult, String> {
    Ok(RecoveryManager::verify_library(db.inner().clone()).await)
}

#[tauri::command]
pub async fn recover_interrupted_jobs(
    db: State<'_, Arc<DbRepository>>,
) -> Result<RecoveryResult, String> {
    Ok(RecoveryManager::recover_interrupted_jobs(&db).await)
}
