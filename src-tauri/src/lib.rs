pub mod commands;
pub mod db;
pub mod engine;
pub mod queue;

use std::sync::Arc;
use tauri::Manager;

pub use commands::{
    cancel_download, cancel_job, cancel_playlist_inspection, enqueue_download, enqueue_playlist_entries,
    force_resume_cooldown, get_app_info, get_engine_status, get_library_jobs, get_queue_jobs,
    inspect_playlist_url, inspect_video_url, pause_queue, resume_queue, set_max_concurrency, start_download,
    validate_url,
};
pub use db::DbRepository;
pub use engine::{DownloadManager, PlaylistInspector};
pub use queue::QueueScheduler;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle();
            let app_data_dir = app_handle
                .path()
                .app_local_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("./data"));
            let db_path = app_data_dir.join("siphonix.db");

            let download_manager = DownloadManager::new();
            let playlist_inspector = PlaylistInspector::new();

            tauri::async_runtime::block_on(async {
                let db = DbRepository::init(&db_path)
                    .await
                    .expect("Failed to initialize SQLite database");
                let db_arc = Arc::new(db);
                let scheduler = QueueScheduler::new(db_arc.clone(), download_manager.clone()).await;

                app.manage(download_manager);
                app.manage(playlist_inspector);
                app.manage(db_arc);
                app.manage(scheduler.clone());

                let main_window = app.get_webview_window("main");
                scheduler.start_worker_loop(main_window);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            validate_url,
            get_app_info,
            get_engine_status,
            inspect_video_url,
            start_download,
            cancel_download,
            enqueue_download,
            pause_queue,
            resume_queue,
            force_resume_cooldown,
            cancel_job,
            set_max_concurrency,
            get_queue_jobs,
            get_library_jobs,
            inspect_playlist_url,
            cancel_playlist_inspection,
            enqueue_playlist_entries
        ])
        .run(tauri::generate_context!())
        .expect("error while running siphonix application");
}
