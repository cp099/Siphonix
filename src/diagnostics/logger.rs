use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use super::event::DiagnosticEvent;

const MAX_LOG_FILE_BYTES: u64 = 5 * 1024 * 1024;
const MAX_LOG_ROTATION_FILES: u32 = 3;

#[derive(Clone)]
pub struct DiagnosticLogger {
    sender: mpsc::Sender<DiagnosticEvent>,
}

impl DiagnosticLogger {
    pub fn new(app_data_dir: &Path) -> Self {
        let log_dir = app_data_dir.join("logs");
        let (tx, rx) = mpsc::channel::<DiagnosticEvent>();

        let dir_clone = log_dir.clone();
        let _ = thread::Builder::new()
            .name("siphonix-diagnostics-logger".to_string())
            .spawn(move || {
                let _ = std::fs::create_dir_all(&dir_clone);
                let log_file_path = dir_clone.join("siphonix_diagnostics.jsonl");

                while let Ok(event) = rx.recv() {
                    if let Ok(line) = serde_json::to_string(&event) {
                        Self::append_log_line(&dir_clone, &log_file_path, &line);
                    }
                }
            });

        Self { sender: tx }
    }

    pub fn log(&self, event: DiagnosticEvent) {
        let _ = self.sender.send(event);
    }

    fn append_log_line(log_dir: &Path, log_file_path: &Path, line: &str) {
        if let Ok(meta) = std::fs::metadata(log_file_path) {
            if meta.len() >= MAX_LOG_FILE_BYTES {
                Self::rotate_logs(log_dir, log_file_path);
            }
        }

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
        {
            let _ = writeln!(file, "{}", line);
        }
    }

    fn rotate_logs(log_dir: &Path, log_file_path: &Path) {
        for i in (1..MAX_LOG_ROTATION_FILES).rev() {
            let src = log_dir.join(format!("siphonix_diagnostics.{}.jsonl", i));
            let dst = log_dir.join(format!("siphonix_diagnostics.{}.jsonl", i + 1));
            if src.exists() {
                let _ = std::fs::rename(&src, &dst);
            }
        }
        let first_rot = log_dir.join("siphonix_diagnostics.1.jsonl");
        if log_file_path.exists() {
            let _ = std::fs::rename(log_file_path, &first_rot);
        }
    }
}
