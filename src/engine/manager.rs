use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

use super::builder::{CommandBuilder, DownloadRequest};
use super::detector::{EngineDetector, EnginePaths};
use super::error::EngineError;
use super::registry::ProcessRegistry;
use super::runner::{OutputParser, ProgressUpdate};

use crate::runtime::EngineManager;

#[derive(Clone)]
pub struct DownloadManager {
    registry: ProcessRegistry,
    engine_paths: Arc<Mutex<Option<EnginePaths>>>,
    runtime_manager: Option<Arc<EngineManager>>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    pub fn new() -> Self {
        let detector_paths = EngineDetector::detect().ok();
        Self {
            registry: ProcessRegistry::new(),
            engine_paths: Arc::new(Mutex::new(detector_paths)),
            runtime_manager: None,
        }
    }

    pub fn new_with_runtime(runtime_manager: Arc<EngineManager>) -> Self {
        Self {
            registry: ProcessRegistry::new(),
            engine_paths: Arc::new(Mutex::new(None)),
            runtime_manager: Some(runtime_manager),
        }
    }

    pub async fn ensure_engine(&self) -> Result<EnginePaths, EngineError> {
        if let Some(ref runtime_mgr) = self.runtime_manager {
            if let Some(paths) = runtime_mgr.get_engine_paths().await {
                return Ok(paths);
            } else {
                return Err(EngineError::EngineNotFound {
                    name: "yt-dlp or ffmpeg".to_string(),
                });
            }
        }

        let mut guard = self.engine_paths.lock().await;
        if let Some(paths) = guard.as_ref() {
            Ok(paths.clone())
        } else {
            let detected = EngineDetector::detect()?;
            *guard = Some(detected.clone());
            Ok(detected)
        }
    }

    pub async fn start_download(
        &self,
        job_id: String,
        request: DownloadRequest,
    ) -> Result<(), EngineError> {
        self.start_download_with_callback(job_id, request, None::<fn(ProgressUpdate)>)
            .await
    }

    pub async fn start_download_with_callback<F>(
        &self,
        job_id: String,
        request: DownloadRequest,
        on_progress: Option<F>,
    ) -> Result<(), EngineError>
    where
        F: Fn(ProgressUpdate) + Send + Sync + 'static,
    {
        let engine = self.ensure_engine().await?;

        let prep_update = ProgressUpdate {
            job_id: job_id.clone(),
            state: "PREPARING".to_string(),
            progress: 0.0,
            speed: None,
            eta: None,
            file_size: None,
            error_message: None,
            final_output_path: None,
        };

        if let Some(ref cb) = on_progress {
            cb(prep_update);
        }

        let (binary, args) = CommandBuilder::build(&request, &engine.yt_dlp, Some(&engine.ffmpeg));

        let mut cmd = Command::new(&binary);
        cmd.args(&args);
        cmd.kill_on_drop(true);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| EngineError::ProcessFailed {
            code: None,
            stderr: e.to_string(),
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| EngineError::ProcessFailed {
                code: None,
                stderr: "Failed to capture process stdout".to_string(),
            })?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| EngineError::ProcessFailed {
                code: None,
                stderr: "Failed to capture process stderr".to_string(),
            })?;

        self.registry.register(job_id.clone(), child).await;

        let registry = self.registry.clone();
        let job_id_clone = job_id.clone();
        let dest_folder = PathBuf::from(request.destination_path.clone());
        let dest_title = request.url.clone();
        let cb_arc: Option<Arc<dyn Fn(ProgressUpdate) + Send + Sync>> =
            on_progress.map(|f| Arc::new(f) as _);

        tokio::spawn(async move {
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            let mut captured_filepath: Option<String> = None;
            let mut stderr_lines = Vec::new();

            while let Ok(Some(line)) = stdout_reader.next_line().await {
                if let Some(path) = OutputParser::extract_filepath(&line) {
                    captured_filepath = Some(path);
                }
                if let Some(mut update) = OutputParser::parse_line(&job_id_clone, &line) {
                    if update.final_output_path.is_none() {
                        update.final_output_path = captured_filepath.clone();
                    }
                    if let Some(ref cb) = cb_arc {
                        cb(update);
                    }
                }
            }

            while let Ok(Some(line)) = stderr_reader.next_line().await {
                if let Some(path) = OutputParser::extract_filepath(&line) {
                    captured_filepath = Some(path);
                }
                stderr_lines.push(line);
            }

            if let Some(mut child_proc) = registry.unregister(&job_id_clone).await {
                let status = child_proc.wait().await;
                match status {
                    Ok(exit_status) if exit_status.success() => {
                        let update = ProgressUpdate {
                            job_id: job_id_clone.clone(),
                            state: "COMPLETED".to_string(),
                            progress: 100.0,
                            speed: None,
                            eta: None,
                            file_size: None,
                            error_message: None,
                            final_output_path: captured_filepath,
                        };
                        if let Some(ref cb) = cb_arc {
                            cb(update);
                        }
                    }
                    Ok(_exit_status) => {
                        let full_stderr = stderr_lines.join("\n");
                        let classified = EngineError::classify_from_stderr(&full_stderr);
                        let error_msg = classified.user_friendly_message();

                        Self::cleanup_partial_files(&dest_folder, &dest_title);

                        let update = ProgressUpdate {
                            job_id: job_id_clone.clone(),
                            state: "FAILED".to_string(),
                            progress: 0.0,
                            speed: None,
                            eta: None,
                            file_size: None,
                            error_message: Some(error_msg),
                            final_output_path: None,
                        };

                        if let Some(ref cb) = cb_arc {
                            cb(update);
                        }
                    }
                    Err(_) => {
                        Self::cleanup_partial_files(&dest_folder, &dest_title);

                        let update = ProgressUpdate {
                            job_id: job_id_clone.clone(),
                            state: "CANCELLED".to_string(),
                            progress: 0.0,
                            speed: None,
                            eta: None,
                            file_size: None,
                            error_message: None,
                            final_output_path: None,
                        };

                        if let Some(ref cb) = cb_arc {
                            cb(update);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn cancel_download(&self, job_id: &str) -> bool {
        self.registry.kill(job_id).await
    }

    pub fn cleanup_partial_files(dest_folder: &Path, title_keyword: &str) {
        if !dest_folder.exists() || !dest_folder.is_dir() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dest_folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let is_partial = file_name.ends_with(".part")
                        || file_name.ends_with(".ytdl")
                        || file_name.ends_with(".temp");

                    if is_partial && (title_keyword.is_empty() || file_name.contains(title_keyword))
                    {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }

        let temp_staging: PathBuf = std::env::temp_dir().join("siphonix_temp");
        if temp_staging.exists() {
            if let Ok(entries) = std::fs::read_dir(&temp_staging) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if title_keyword.is_empty() || file_name.contains(title_keyword) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
}
