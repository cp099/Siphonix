use std::process::Stdio;
use std::sync::Arc;
use tauri::{Emitter, WebviewWindow};
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
        window: WebviewWindow,
        job_id: String,
        request: DownloadRequest,
    ) -> Result<(), EngineError> {
        let engine = self.ensure_engine().await?;

        // 1. Emit PREPARING state
        let _ = window.emit(
            "download-progress",
            ProgressUpdate {
                job_id: job_id.clone(),
                state: "PREPARING".to_string(),
                progress: 0.0,
                speed: None,
                eta: None,
                file_size: None,
                error_message: None,
                final_output_path: None,
            },
        );

        // 2. Build structured args
        let args = CommandBuilder::build_args(&request, &engine);

        // 3. Spawn tokio process directly
        let mut cmd = Command::new(&engine.yt_dlp);
        cmd.args(&args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| EngineError::ProcessFailed {
            code: None,
            stderr: e.to_string(),
        })?;

        let stdout = child.stdout.take().ok_or_else(|| EngineError::ProcessFailed {
            code: None,
            stderr: "Failed to capture process stdout".to_string(),
        })?;
        let stderr = child.stderr.take().ok_or_else(|| EngineError::ProcessFailed {
            code: None,
            stderr: "Failed to capture process stderr".to_string(),
        })?;

        // 4. Register process in ProcessRegistry
        self.registry.register(job_id.clone(), child).await;

        let registry = self.registry.clone();
        let job_id_clone = job_id.clone();

        // 5. Asynchronously monitor process and stream progress
        tokio::spawn(async move {
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            let mut captured_filepath: Option<String> = None;
            let mut stderr_lines = Vec::new();

            while let Ok(Some(line)) = stdout_reader.next_line().await {
                if OutputParser::is_filepath_line(&line) {
                    captured_filepath = Some(line.trim().to_string());
                } else if let Some(update) = OutputParser::parse_line(&job_id_clone, &line) {
                    let _ = window.emit("download-progress", update);
                }
            }

            while let Ok(Some(line)) = stderr_reader.next_line().await {
                stderr_lines.push(line);
            }

            // Unregister child process handle
            if let Some(mut child_proc) = registry.unregister(&job_id_clone).await {
                let status = child_proc.wait().await;
                match status {
                    Ok(exit_status) if exit_status.success() => {
                        let _ = window.emit(
                            "download-progress",
                            ProgressUpdate {
                                job_id: job_id_clone.clone(),
                                state: "COMPLETED".to_string(),
                                progress: 100.0,
                                speed: None,
                                eta: None,
                                file_size: None,
                                error_message: None,
                                final_output_path: captured_filepath,
                            },
                        );
                    }
                    Ok(_exit_status) => {
                        let full_stderr = stderr_lines.join("\n");
                        let classified = EngineError::classify_from_stderr(&full_stderr);
                        let error_msg = classified.to_string();

                        let _ = window.emit(
                            "download-progress",
                            ProgressUpdate {
                                job_id: job_id_clone.clone(),
                                state: "FAILED".to_string(),
                                progress: 0.0,
                                speed: None,
                                eta: None,
                                file_size: None,
                                error_message: Some(error_msg),
                                final_output_path: None,
                            },
                        );
                    }
                    Err(_) => {
                        let _ = window.emit(
                            "download-progress",
                            ProgressUpdate {
                                job_id: job_id_clone.clone(),
                                state: "CANCELLED".to_string(),
                                progress: 0.0,
                                speed: None,
                                eta: None,
                                file_size: None,
                                error_message: None,
                                final_output_path: None,
                            },
                        );
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn cancel_download(&self, job_id: &str) -> bool {
        self.registry.kill(job_id).await
    }
}
