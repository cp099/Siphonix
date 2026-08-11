use std::sync::Arc;
use tauri::State;

use crate::db::repository::{DbLibraryItem, DbRepository};
use crate::library::LibraryService;

#[tauri::command]
pub async fn get_library_items(
    db: State<'_, Arc<DbRepository>>,
    search: Option<String>,
    filter_mode: Option<String>,
    filter_status: Option<String>,
    sort_by: Option<String>,
) -> Result<Vec<DbLibraryItem>, String> {
    let service = LibraryService::new(db.inner().clone());
    service.get_library_items(search, filter_mode, filter_status, sort_by).await
}

#[tauri::command]
pub async fn verify_library_status(
    db: State<'_, Arc<DbRepository>>,
) -> Result<Vec<DbLibraryItem>, String> {
    let service = LibraryService::new(db.inner().clone());
    service.verify_library_items().await
}

#[tauri::command]
pub async fn open_library_item(
    db: State<'_, Arc<DbRepository>>,
    item_id: String,
) -> Result<(), String> {
    let service = LibraryService::new(db.inner().clone());
    service.open_item(&item_id).await
}

#[tauri::command]
pub async fn reveal_library_item(
    db: State<'_, Arc<DbRepository>>,
    item_id: String,
) -> Result<(), String> {
    let service = LibraryService::new(db.inner().clone());
    service.reveal_item(&item_id).await
}

#[tauri::command]
pub async fn remove_library_item(
    db: State<'_, Arc<DbRepository>>,
    item_id: String,
) -> Result<(), String> {
    let service = LibraryService::new(db.inner().clone());
    service.remove_item_record(&item_id).await
}

#[tauri::command]
pub async fn delete_library_file(
    db: State<'_, Arc<DbRepository>>,
    item_id: String,
) -> Result<(), String> {
    let service = LibraryService::new(db.inner().clone());
    service.delete_item_file(&item_id).await
}
