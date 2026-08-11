use std::sync::Arc;
use tauri::State;

use crate::runtime::{EngineManager, RuntimeStatus};

#[tauri::command]
pub async fn get_runtime_status(
    engine_mgr: State<'_, Arc<EngineManager>>,
) -> Result<RuntimeStatus, String> {
    Ok(engine_mgr.get_status().await)
}

#[tauri::command]
pub async fn refresh_runtime_status(
    engine_mgr: State<'_, Arc<EngineManager>>,
) -> Result<RuntimeStatus, String> {
    Ok(engine_mgr.refresh_status().await)
}
