use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::db::repository::DbRepository;
use crate::engine::builder::{CommandBuilder, DownloadRequest};
use crate::engine::detector::EngineDetector;
use crate::engine::error::EngineError;
use crate::engine::manager::DownloadManager;
use crate::engine::options::DownloadOptions;
use crate::engine::runner::OutputParser;
use crate::engine::url::validate_url;
use crate::library::LibraryService;
use crate::queue::job::DownloadJob;
use crate::queue::state::JobState;
use crate::tui::wizard::run_interactive_wizard;
use chrono::Utc;

#[derive(Parser, Debug)]
#[command(
    name = "siphonix",
    version = env!("CARGO_PKG_VERSION"),
    about = "Universal Media Downloader (CLI & Terminal UI)",
    long_about = "A high-performance media downloader for videos, audio, and playlists powered by Rust, yt-dlp, and FFmpeg."
)]
pub struct CliArgs {
    #[arg(value_name = "URL", help = "URL to download (video or playlist)")]
    pub url: Option<String>,

    #[arg(short = 'm', long = "mode", value_parser = ["video", "audio"])]
    pub mode: Option<String>,

    #[arg(short = 'f', long = "format")]
    pub format: Option<String>,

    #[arg(short = 'q', long = "quality")]
    pub quality: Option<String>,

    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    #[arg(long = "embed-thumbnail", default_value_t = true)]
    pub embed_thumbnail: bool,

    #[arg(long = "embed-metadata", default_value_t = true)]
    pub embed_metadata: bool,

    #[arg(long = "history", short = 'l')]
    pub history: bool,

    #[arg(long = "diagnostics", short = 'd')]
    pub diagnostics: bool,
}

pub async fn run_cli() {
    let args = CliArgs::parse();

    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let app_data_dir = PathBuf::from(home_dir).join(".siphonix");
    let _ = std::fs::create_dir_all(&app_data_dir);
    let db_path = app_data_dir.join("siphonix.db");

    let db = match DbRepository::init(&db_path).await {
        Ok(d) => Arc::new(d),
        Err(e) => {
            eprintln!(
                "{} Failed to initialize local database: {}",
                "Warning:".yellow().bold(),
                e
            );
            eprintln!("Continuing with temporary storage.\n");
            let temp_db =
                std::env::temp_dir().join(format!("siphonix_temp_{}.db", std::process::id()));
            match DbRepository::init(&temp_db).await {
                Ok(tdb) => Arc::new(tdb),
                Err(_) => {
                    eprintln!("Fatal error: Could not initialize storage.");
                    return;
                }
            }
        }
    };

    if args.diagnostics {
        clear_screen();
        print_banner();
        run_diagnostics().await;
        return;
    }

    if args.history {
        clear_screen();
        print_banner();
        run_history(db).await;
        return;
    }

    if let Some(ref url) = args.url {
        clear_screen();
        print_banner();
        run_direct_download(url, &args, db).await;
    } else {
        run_interactive_wizard(db).await;
    }
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

pub fn print_banner() {
    println!();
    println!(
        "{}",
        "  ███████╗██╗██████╗ ██╗  ██╗ ██████╗ ███╗   ██╗██╗██╗  ██╗"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ██╔════╝██║██╔══██╗██║  ██║██╔═══██╗████╗  ██║██║╚██╗██╔╝"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ███████╗██║██████╔╝███████║██║   ██║██╔██╗ ██║██║ ╚███╔╝ "
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ╚════██║██║██╔═══╝ ██╔══██║██║   ██║██║╚██╗██║██║ ██╔██╗ "
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ███████║██║██║     ██║  ██║╚██████╔╝██║ ╚████║██║██╔╝ ██╗"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ╚══════╝╚═╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝"
            .cyan()
            .bold()
    );
    println!();
    println!(
        "   {} {} — {}",
        "Siphonix".white().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).cyan(),
        "Universal Terminal Media Downloader".bright_black()
    );
    println!(
        "   {} {} ({}) • {}",
        "Developed by:".bright_black(),
        "Chirag P Patil".white().bold(),
        "cp099".cyan(),
        "https://github.com/cp099".bright_black()
    );
    println!();
}

pub async fn run_diagnostics() {
    println!(
        "{}",
        "═════════════════════════════════════════════════════════════════".bright_black()
    );
    println!(
        "  {}",
        "Siphonix System & Engine Diagnostics".white().bold()
    );
    println!(
        "{}",
        "═════════════════════════════════════════════════════════════════\n".bright_black()
    );

    match EngineDetector::detect() {
        Ok(paths) => {
            println!(
                "  {} yt-dlp binary:  {}",
                "✓".green().bold(),
                paths.yt_dlp.display().to_string().cyan()
            );
            println!(
                "  {} ffmpeg binary:  {}",
                "✓".green().bold(),
                paths.ffmpeg.display().to_string().cyan()
            );

            let yt_ver = Command::new(&paths.yt_dlp)
                .arg("--version")
                .output()
                .await
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_else(|| "Unknown".to_string());

            let ff_ver = Command::new(&paths.ffmpeg)
                .arg("-version")
                .output()
                .await
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| s.lines().next().map(|l| l.to_string()))
                .unwrap_or_else(|| "Unknown".to_string());

            println!(
                "  {} yt-dlp version: {}",
                "✓".green().bold(),
                yt_ver.trim().white()
            );
            println!(
                "  {} ffmpeg version: {}",
                "✓".green().bold(),
                ff_ver.trim().white()
            );
            println!(
                "\n  Engine Health: {} (All dependencies operational)",
                "READY".green().bold()
            );
        }
        Err(e) => {
            println!("  {} Engine Detection Failed: {}", "✗".red().bold(), e);
            println!(
                "\n  Engine Health: {} (Please install yt-dlp and ffmpeg)",
                "DEGRADED".red().bold()
            );
        }
    }
    println!(
        "{}",
        "═════════════════════════════════════════════════════════════════\n".bright_black()
    );
}

pub async fn run_history(db: Arc<DbRepository>) {
    println!(
        "{}",
        "═════════════════════════════════════════════════════════════════".bright_black()
    );
    println!("  {}", "Siphonix Download History".white().bold());
    println!(
        "{}",
        "═════════════════════════════════════════════════════════════════\n".bright_black()
    );

    let lib_svc = LibraryService::new(db);
    match lib_svc.get_library_items(None, None, None, None).await {
        Ok(items) if items.is_empty() => {
            println!("  {}\n", "No downloads in history yet.".bright_black());
        }
        Ok(items) => {
            for (idx, item) in items.iter().enumerate() {
                let exists = Path::new(&item.file_path).exists();
                let status_badge = if exists {
                    "AVAILABLE".green()
                } else {
                    "MISSING".red()
                };
                let size_mb = (item.file_size_bytes as f64) / (1024.0 * 1024.0);

                println!(
                    "  {}. {} [{}]",
                    (idx + 1).to_string().cyan(),
                    item.title.white().bold(),
                    item.format.yellow()
                );
                println!(
                    "     Path: {} ({:.2} MB) • Status: {}",
                    item.file_path.bright_black(),
                    size_mb,
                    status_badge
                );
                println!("     Date: {}\n", item.created_at.bright_black());
            }
        }
        Err(e) => {
            println!("  {} Failed to query history: {}", "Error:".red().bold(), e);
        }
    }
    println!(
        "{}",
        "═════════════════════════════════════════════════════════════════\n".bright_black()
    );
}

pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

pub fn prompt_numbered_choice(prompt: &str, options: &[&str], default_idx: usize) -> usize {
    println!("  {}", prompt.white().bold());
    for (idx, opt) in options.iter().enumerate() {
        println!("   {} {}", format!("[{}]", idx + 1).cyan().bold(), opt);
    }
    println!();

    loop {
        let input = match inquire::Text::new(&format!("  Select Option [1-{}]:", options.len()))
            .with_default(&(default_idx + 1).to_string())
            .prompt()
        {
            Ok(val) => val.trim().to_string(),
            Err(_) => return default_idx,
        };

        if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= options.len() {
                return num - 1;
            }
        }
        println!("  {}", "Invalid selection, please choose a valid number.".yellow());
    }
}

async fn run_direct_download(url: &str, args: &CliArgs, db: Arc<DbRepository>) {
    let val = validate_url(url);
    if !val.valid {
        eprintln!("\n{} {}\n", "Invalid URL:".red().bold(), val.message);
        return;
    }

    let engine = match EngineDetector::detect() {
        Ok(e) => e,
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            return;
        }
    };

    let mode = args.mode.clone().unwrap_or_else(|| {
        if let Some(ref f) = args.format {
            let fl = f.to_lowercase();
            if fl == "mp3" || fl == "flac" || fl == "m4a" || fl == "wav" || fl == "opus" {
                "audio".to_string()
            } else {
                "video".to_string()
            }
        } else {
            "video".to_string()
        }
    });

    let format = args.format.clone().unwrap_or_else(|| {
        if mode == "audio" {
            "MP3".to_string()
        } else {
            "MP4".to_string()
        }
    });

    let quality = args.quality.clone().unwrap_or_else(|| {
        if mode == "audio" {
            "320k".to_string()
        } else {
            "1080p".to_string()
        }
    });

    let default_dest = "~/Downloads/Siphonix";
    let target_dest = args
        .output
        .clone()
        .unwrap_or_else(|| default_dest.to_string());
    let expanded_dest = CommandBuilder::expand_tilde(&target_dest);
    let _ = std::fs::create_dir_all(&expanded_dest);

    let mut opts = DownloadOptions {
        media_mode: mode.clone(),
        video: crate::engine::options::VideoOptions {
            resolution: if mode == "video" {
                quality.clone()
            } else {
                "1080p".to_string()
            },
            ..Default::default()
        },
        audio: crate::engine::options::AudioOptions {
            format: if mode == "audio" {
                format.to_lowercase()
            } else {
                "mp3".to_string()
            },
            quality: if mode == "audio" {
                quality.clone()
            } else {
                "320k".to_string()
            },
            ..Default::default()
        },
        output: crate::engine::options::OutputOptions {
            destination_path: expanded_dest.clone(),
            container: format.to_lowercase(),
            ..Default::default()
        },
        metadata: crate::engine::options::MetadataOptions {
            embed_metadata: args.embed_metadata,
            embed_thumbnail: args.embed_thumbnail,
            ..Default::default()
        },
        ..Default::default()
    };

    if let Err(err) = opts.validate() {
        eprintln!("\n{} {}\n", "Invalid Options:".red().bold(), err);
        return;
    }

    let job_id = format!("job_{}", Utc::now().timestamp_millis());

    println!(
        "{}",
        "  ═════════════════════════════════════════════════════════════".bright_black()
    );
    println!("  {}", "Siphonix Download Status & Progress".white().bold());
    println!(
        "{}",
        "  ═════════════════════════════════════════════════════════════".bright_black()
    );
    println!(
        "    {}    {}",
        "Media:   ".bright_black(),
        truncate_str(url, 48).white().bold()
    );
    println!(
        "    {}   {} ({})",
        "Format:  ".bright_black(),
        format.yellow().bold(),
        quality.cyan()
    );
    println!(
        "    {}  {}",
        "Save To: ".bright_black(),
        expanded_dest.cyan()
    );
    println!(
        "{}",
        "  ═════════════════════════════════════════════════════════════\n".bright_black()
    );

    let request = DownloadRequest {
        url: url.to_string(),
        media_mode: mode.clone(),
        audio_format: if mode == "audio" {
            Some(format.clone())
        } else {
            None
        },
        audio_quality: if mode == "audio" {
            Some(quality.clone())
        } else {
            None
        },
        video_format: if mode == "video" {
            Some(format.clone())
        } else {
            None
        },
        video_quality: if mode == "video" {
            Some(quality.clone())
        } else {
            None
        },
        destination_path: expanded_dest.clone(),
        options: Some(opts.clone()),
    };

    let (binary, cli_args) = CommandBuilder::build(&request, &engine.yt_dlp, Some(&engine.ffmpeg));

    let pb = ProgressBar::new(100);
    let pb_style = ProgressStyle::default_bar()
        .template("  {bar:35.cyan/blue} {percent:>3}% | {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("━╸─");
    pb.set_style(pb_style);
    pb.set_message("Starting download stream...");

    let start_time = Instant::now();

    let mut cmd = Command::new(&binary);
    cmd.args(&cli_args);
    cmd.kill_on_drop(true);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            pb.finish_and_clear();
            eprintln!(
                "\n{} Failed to spawn process: {}\n",
                "Download Failed:".red().bold(),
                e
            );
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            pb.finish_and_clear();
            eprintln!(
                "\n{} Failed to capture process stdout stream\n",
                "Error:".red().bold()
            );
            return;
        }
    };
    let stderr = match child.stderr.take() {
        Some(s) => s,
        None => {
            pb.finish_and_clear();
            eprintln!(
                "\n{} Failed to capture process stderr stream\n",
                "Error:".red().bold()
            );
            return;
        }
    };

    let pb_clone = pb.clone();
    let j_id_copy = job_id.clone();
    let monitor_handle = tokio::spawn(async move {
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut captured_filepath: Option<String> = None;
        let mut stderr_buf = Vec::new();

        let mut stdout_done = false;
        let mut stderr_done = false;

        while !stdout_done || !stderr_done {
            tokio::select! {
                line_res = stdout_reader.next_line(), if !stdout_done => {
                    match line_res {
                        Ok(Some(line)) => {
                            if let Some(path) = OutputParser::extract_filepath(&line) {
                                captured_filepath = Some(path);
                            }
                            if let Some(update) = OutputParser::parse_line(&j_id_copy, &line) {
                                pb_clone.set_position(update.progress as u64);
                                let speed_str = update.speed.unwrap_or_else(|| "-- MB/s".to_string());
                                let eta_str = update.eta.unwrap_or_else(|| "--:--".to_string());
                                pb_clone.set_message(format!("{} | ETA: {}", speed_str, eta_str));
                            } else if line.contains("[Merger]") || line.contains("[ExtractAudio]") {
                                pb_clone.set_message("Processing & multiplexing container...");
                            } else if line.contains("[ThumbnailsConvertor]") || line.contains("[Metadata]") {
                                pb_clone.set_message("Embedding artwork & metadata tags...");
                            }
                        }
                        Ok(None) => { stdout_done = true; }
                        Err(_) => { stdout_done = true; }
                    }
                }
                line_res = stderr_reader.next_line(), if !stderr_done => {
                    match line_res {
                        Ok(Some(line)) => {
                            if let Some(path) = OutputParser::extract_filepath(&line) {
                                captured_filepath = Some(path);
                            }
                            if let Some(update) = OutputParser::parse_line(&j_id_copy, &line) {
                                pb_clone.set_position(update.progress as u64);
                                let speed_str = update.speed.unwrap_or_else(|| "-- MB/s".to_string());
                                let eta_str = update.eta.unwrap_or_else(|| "--:--".to_string());
                                pb_clone.set_message(format!("{} | ETA: {}", speed_str, eta_str));
                            }
                            stderr_buf.push(line);
                        }
                        Ok(None) => { stderr_done = true; }
                        Err(_) => { stderr_done = true; }
                    }
                }
            }
        }
        (captured_filepath, stderr_buf)
    });

    let status = tokio::select! {
        res = child.wait() => res,
        _ = tokio::signal::ctrl_c() => {
            let _ = child.start_kill();
            pb.finish_and_clear();
            DownloadManager::cleanup_partial_files(Path::new(&expanded_dest), "");
            eprintln!("\n{}\n", "Download cancelled by user.".yellow().bold());
            return;
        }
    };

    let (final_filepath, stderr_buf) = monitor_handle.await.unwrap_or((None, Vec::new()));

    pb.finish_and_clear();

    match status {
        Ok(s) if s.success() && final_filepath.is_some() => {
            if let Some(path_str) = final_filepath {
                let p = Path::new(&path_str);
                let file_size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                let size_mb = (file_size as f64) / (1024.0 * 1024.0);
                let elapsed_secs = start_time.elapsed().as_secs_f32();

                println!(
                    "{}",
                    "  ╔═════════════════════════════════════════════════════════════╗".green()
                );
                println!(
                    "  ║  {}                                 ║",
                    "DOWNLOAD COMPLETED SUCCESSFULLY!".green().bold()
                );
                println!(
                    "{}",
                    "  ╠═════════════════════════════════════════════════════════════╣".green()
                );
                println!(
                    "  ║  {} {}",
                    "File:   ".bright_black(),
                    truncate_str(&p.file_name().unwrap_or_default().to_string_lossy(), 46)
                        .white()
                        .bold()
                );
                println!(
                    "  ║  {} {}",
                    "Saved:  ".bright_black(),
                    truncate_str(&p.display().to_string(), 46).cyan()
                );
                println!(
                    "  ║  {} {:.2} MB • {} {:.1}s",
                    "Size:   ".bright_black(),
                    size_mb,
                    "Elapsed:".bright_black(),
                    elapsed_secs
                );
                println!(
                    "{}",
                    "  ╚═════════════════════════════════════════════════════════════╝".green()
                );

                let download_job = DownloadJob {
                    id: job_id.clone(),
                    url: url.to_string(),
                    title: p
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    thumbnail_url: None,
                    media_mode: mode.to_string(),
                    format: format.to_string(),
                    quality: quality.to_string(),
                    destination_path: expanded_dest.to_string(),
                    state: JobState::COMPLETED,
                    progress: 100.0,
                    download_speed: None,
                    eta: None,
                    file_size: Some(format!("{:.2} MB", size_mb)),
                    error_message: None,
                    last_error_category: None,
                    retry_count: 0,
                    max_retries: 5,
                    next_retry_at: None,
                    created_at: Utc::now(),
                    started_at: Some(Utc::now()),
                    completed_at: Some(Utc::now()),
                    source_video_id: None,
                    source_playlist_id: None,
                    source_playlist_title: None,
                    playlist_entry_index: None,
                    options: opts,
                };

                let _ = db.insert_job(&download_job.to_db_job()).await;
                let lib_svc = LibraryService::new(db);
                let _ = lib_svc
                    .register_completed_job(&download_job, Some(&path_str))
                    .await;
            }
        }
        Ok(_) => {
            let stderr_text = stderr_buf.join("\n");
            let classified = EngineError::classify_from_stderr(&stderr_text);
            DownloadManager::cleanup_partial_files(Path::new(&expanded_dest), "");
            eprintln!(
                "\n{} {}\n",
                "Download Failed:".red().bold(),
                classified.user_friendly_message()
            );
        }
        Err(e) => {
            DownloadManager::cleanup_partial_files(Path::new(&expanded_dest), "");
            eprintln!(
                "\n{} Failed to wait on child process: {}\n",
                "Download Failed:".red().bold(),
                e
            );
        }
    }
}
