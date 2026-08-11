use std::sync::Arc;
use tauri::State;
use chrono::Utc;

use crate::db::repository::{DbRepository, DbPreset};
use crate::engine::options::{DownloadOptions, DownloadPreset};

fn rand_preset_suffix() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let s: String = (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + (idx - 10)) as char
            }
        })
        .collect();
    s
}

#[tauri::command]
pub async fn validate_download_options(
    mut options: DownloadOptions,
) -> Result<DownloadOptions, String> {
    options.validate().map_err(|e| e.to_string())?;
    Ok(options)
}

#[tauri::command]
pub async fn get_presets(
    db: State<'_, Arc<DbRepository>>,
) -> Result<Vec<DownloadPreset>, String> {
    let db_presets = db.get_presets().await.map_err(|e| e.to_string())?;
    let mut presets = Vec::new();
    for p in db_presets {
        if let Ok(opts) = serde_json::from_str::<DownloadOptions>(&p.options_json) {
            presets.push(DownloadPreset {
                id: p.id,
                name: p.name,
                description: p.description,
                is_default: p.is_default != 0,
                options: opts,
                created_at: p.created_at,
                updated_at: p.updated_at,
            });
        }
    }
    Ok(presets)
}

#[tauri::command]
pub async fn save_preset(
    preset_id: Option<String>,
    name: String,
    description: Option<String>,
    is_default: bool,
    options: DownloadOptions,
    db: State<'_, Arc<DbRepository>>,
) -> Result<DownloadPreset, String> {
    let now = Utc::now().to_rfc3339();
    let id = preset_id.unwrap_or_else(|| format!("preset-{}-{}", Utc::now().timestamp_millis(), rand_preset_suffix()));
    let options_json = serde_json::to_string(&options).map_err(|e| e.to_string())?;

    let db_preset = DbPreset {
        id: id.clone(),
        name: name.clone(),
        description: description.clone(),
        is_default: if is_default { 1 } else { 0 },
        options_json,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    db.insert_preset(&db_preset).await.map_err(|e| e.to_string())?;

    if is_default {
        db.set_default_preset(&id).await.map_err(|e| e.to_string())?;
    }

    Ok(DownloadPreset {
        id,
        name,
        description,
        is_default,
        options,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn delete_preset(
    preset_id: String,
    db: State<'_, Arc<DbRepository>>,
) -> Result<(), String> {
    db.delete_preset(&preset_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_default_preset(
    preset_id: String,
    db: State<'_, Arc<DbRepository>>,
) -> Result<(), String> {
    db.set_default_preset(&preset_id).await.map_err(|e| e.to_string())
}
