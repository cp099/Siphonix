use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use super::event::DiagnosticEvent;

const MAX_LOG_FILE_BYTES: u64 = 5 * 1024 * 1024; // 5 MB max
const MAX_LOG_ROTATION_FILES: u32 = 3;

#[derive(Clone)]
pub struct DiagnosticLogger {
    log_dir: PathBuf,
    sender: mpsc::UnboundedSender<DiagnosticEvent>,
}

impl DiagnosticLogger {
    pub fn new(app_data_dir: &Path) -> Self {
        let log_dir = app_data_dir.join("logs");
        let (tx, mut rx) = mpsc::unbounded_channel::<DiagnosticEvent>();

        let dir_clone = log_dir.clone();
        tokio::spawn(async move {
            let _ = tokio::fs::create_dir_all(&dir_clone).await;
            let log_file_path = dir_clone.join("siphonix_diagnostics.jsonl");

            while let Some(event) = rx.recv().await {
                if let Ok(line) = serde_json::to_string(&event) {
                    Self::append_log_line(&dir_clone, &log_file_path, &line).await;
                }
            }
        });

        Self { log_dir, sender: tx }
    }

    pub fn log(&self, event: DiagnosticEvent) {
        // Non-blocking fire-and-forget sending over Tokio channel
        let _ = self.sender.send(event);
    }

    async fn append_log_line(log_dir: &Path, log_file_path: &Path, line: &str) {
        // Check size & rotate if exceeding 5 MB
        if let Ok(meta) = tokio::fs::metadata(log_file_path).await {
            if meta.len() >= MAX_LOG_FILE_BYTES {
                Self::rotate_logs(log_dir, log_file_path).await;
            }
        }

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .await
        {
            let line_with_newline = format!("{}\n", line);
            let _ = file.write_all(line_with_newline.as_bytes()).await;
        }
    }

    async fn rotate_logs(log_dir: &Path, log_file_path: &Path) {
        for i in (1..MAX_LOG_ROTATION_FILES).rev() {
            let src = log_dir.join(format!("siphonix_diagnostics.{}.jsonl", i));
            let dst = log_dir.join(format!("siphonix_diagnostics.{}.jsonl", i + 1));
            if src.exists() {
                let _ = tokio::fs::rename(&src, &dst).await;
            }
        }
        let first_rot = log_dir.join("siphonix_diagnostics.1.jsonl");
        if log_file_path.exists() {
            let _ = tokio::fs::rename(log_file_path, &first_rot).await;
        }
    }
}
