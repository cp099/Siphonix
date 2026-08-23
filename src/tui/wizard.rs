use chrono::Utc;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::cli::{
    clear_screen, print_banner, prompt_numbered_choice, run_diagnostics, run_history, truncate_str,
};
use crate::db::DbRepository;
use crate::engine::builder::{CommandBuilder, DownloadRequest};
use crate::engine::detector::{EngineDetector, EnginePaths};
use crate::engine::error::EngineError;
use crate::engine::info::inspect_url;
use crate::engine::manager::DownloadManager;
use crate::engine::options::DownloadOptions;
use crate::engine::runner::OutputParser;
use crate::engine::url::validate_url;
use crate::library::LibraryService;
use crate::queue::job::DownloadJob;
use crate::queue::state::JobState;

pub async fn run_interactive_wizard(db: Arc<DbRepository>) {
    let engine = match EngineDetector::detect() {
        Ok(e) => e,
        Err(err) => {
            clear_screen();
            print_banner();
            eprintln!("{} {}\n", "Error:".red().bold(), err);
            eprintln!("Please ensure yt-dlp and ffmpeg are installed on your PATH.");
            eprintln!("macOS:   brew install yt-dlp ffmpeg");
            eprintln!("Windows: winget install yt-dlp Gyan.FFmpeg\n");
            return;
        }
    };

    loop {
        clear_screen();
        print_banner();

        println!("  {}", "Main Menu:".white().bold());
        println!(
            "   {} Paste / Enter YouTube Video or Playlist URL",
            "[1]".green().bold()
        );
        println!("   {} View Download History", "[2]".cyan().bold());
        println!("   {} System & Engine Diagnostics", "[3]".cyan().bold());
        println!("   {} Exit Siphonix\n", "[4]".bright_black());

        let url_or_opt = match Text::new("  Enter URL or Option [1-4]:")
            .with_placeholder("https://www.youtube.com/watch?v=... or 1")
            .prompt()
        {
            Ok(u) => u.trim().to_string(),
            Err(_) => break,
        };

        if url_or_opt.is_empty() {
            continue;
        }

        if url_or_opt == "4"
            || url_or_opt.eq_ignore_ascii_case("exit")
            || url_or_opt.eq_ignore_ascii_case("q")
        {
            println!(
                "\n  {}\n",
                "Thank you for using Siphonix. Goodbye!".cyan().bold()
            );
            break;
        }

        if url_or_opt == "2"
            || url_or_opt.eq_ignore_ascii_case("history")
            || url_or_opt.eq_ignore_ascii_case("h")
        {
            clear_screen();
            print_banner();
            run_history(db.clone()).await;
            let _ = Text::new("  Press Enter to return to main menu...").prompt();
            continue;
        }

        if url_or_opt == "3"
            || url_or_opt.eq_ignore_ascii_case("diagnostics")
            || url_or_opt.eq_ignore_ascii_case("d")
        {
            clear_screen();
            print_banner();
            run_diagnostics().await;
            let _ = Text::new("  Press Enter to return to main menu...").prompt();
            continue;
        }

        let target_url = if url_or_opt == "1" {
            match Text::new("  Paste YouTube URL:")
                .with_placeholder("https://www.youtube.com/watch?v=...")
                .prompt()
            {
                Ok(u) => u.trim().to_string(),
                Err(_) => continue,
            }
        } else {
            url_or_opt
        };

        if target_url.is_empty() {
            continue;
        }

        let val = validate_url(&target_url);
        if !val.valid {
            eprintln!("\n  {} {}\n", "Invalid URL:".red().bold(), val.message);
            let _ = Text::new("  Press Enter to continue...").prompt();
            continue;
        }

        let spinner = ProgressBar::new_spinner();
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("  {spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner());
        spinner.set_style(spinner_style);
        spinner.set_message("Inspecting media metadata...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let info = match inspect_url(&target_url, &engine).await {
            Ok(i) => {
                spinner.finish_and_clear();
                i
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!(
                    "\n  {} Failed to inspect URL: {}\n",
                    "Error:".red().bold(),
                    e
                );
                let _ = Text::new("  Press Enter to continue...").prompt();
                continue;
            }
        };

        let duration_str = info
            .duration
            .map(|d| {
                let m = d / 60;
                let s = d % 60;
                format!("{}m {:02}s", m, s)
            })
            .unwrap_or_else(|| "Unknown".to_string());

        clear_screen();
        print_banner();

        println!(
            "  {}",
            "┌─────────────────────────────────────────────────────────────┐".cyan()
        );
        println!(
            "  │  {} {}",
            "Title:   ".bright_black(),
            truncate_str(&info.title, 47).white().bold()
        );
        println!(
            "  │  {} {}",
            "Uploader:".bright_black(),
            truncate_str(info.uploader.as_deref().unwrap_or("Unknown"), 47).cyan()
        );
        println!(
            "  │  {} {}",
            "Duration:".bright_black(),
            duration_str.yellow()
        );
        println!(
            "  {}",
            "└─────────────────────────────────────────────────────────────┘\n".cyan()
        );

        let format_options = [
            "MP4 Video  (Universal / QuickTime / Apple / Windows)",
            "MP3 Audio  (Universal / Best for Music & Podcasts)",
            "MKV Video  (High Fidelity / Preserves all tracks & subtitles)",
            "FLAC Audio (Lossless Studio Audio Quality)",
            "M4A Audio  (Apple AAC High Efficiency)",
        ];

        let fmt_idx = prompt_numbered_choice("Select Download Format:", &format_options, 0);

        let (is_video, format_name) = match fmt_idx {
            0 => (true, "MP4"),
            1 => (false, "MP3"),
            2 => (true, "MKV"),
            3 => (false, "FLAC"),
            _ => (false, "M4A"),
        };

        clear_screen();
        print_banner();

        println!(
            "  {}",
            "┌─────────────────────────────────────────────────────────────┐".cyan()
        );
        println!(
            "  │  Selected: {}",
            format!("{} ({})", info.title, format_name).white().bold()
        );
        println!(
            "  {}",
            "└─────────────────────────────────────────────────────────────┘\n".cyan()
        );

        let quality_name = if is_video {
            let video_qualities = [
                "Best Available",
                "4K  (2160p Ultra HD)",
                "2K  (1440p Quad HD)",
                "1080p (Full HD)",
                "720p  (HD)",
                "480p  (Standard)",
                "360p  (Low)",
            ];
            let q_idx = prompt_numbered_choice("Select Video Quality:", &video_qualities, 3);
            match q_idx {
                0 => "best",
                1 => "2160p",
                2 => "1440p",
                3 => "1080p",
                4 => "720p",
                5 => "480p",
                _ => "360p",
            }
        } else if format_name == "FLAC" {
            "best"
        } else {
            let audio_qualities = [
                "320 kbps (High Quality)",
                "256 kbps (Very Good)",
                "192 kbps (Standard)",
                "128 kbps (Compact)",
                "Best VBR (Auto Variable Bitrate)",
            ];
            let q_idx = prompt_numbered_choice("Select Audio Bitrate:", &audio_qualities, 0);
            match q_idx {
                0 => "320k",
                1 => "256k",
                2 => "192k",
                3 => "128k",
                _ => "0",
            }
        };

        clear_screen();
        print_banner();

        let default_dest = "~/Downloads/Siphonix";
        let chosen_dest = match Text::new("  Download Destination Folder:")
            .with_default(default_dest)
            .prompt()
        {
            Ok(d) => d,
            Err(_) => default_dest.to_string(),
        };
        let expanded_dest = CommandBuilder::expand_tilde(&chosen_dest);
        let _ = std::fs::create_dir_all(&expanded_dest);

        clear_screen();
        print_banner();

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
            truncate_str(&info.title, 48).white().bold()
        );
        println!(
            "    {}   {} ({})",
            "Format:  ".bright_black(),
            format_name.yellow().bold(),
            quality_name.cyan()
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

        let job_id = format!("job_{}", Utc::now().timestamp_millis());
        let mode_str = if is_video { "video" } else { "audio" };

        let opts = DownloadOptions {
            media_mode: mode_str.to_string(),
            video: crate::engine::options::VideoOptions {
                resolution: if is_video {
                    quality_name.to_string()
                } else {
                    "1080p".to_string()
                },
                ..Default::default()
            },
            audio: crate::engine::options::AudioOptions {
                format: if !is_video {
                    format_name.to_lowercase()
                } else {
                    "mp3".to_string()
                },
                quality: if !is_video {
                    quality_name.to_string()
                } else {
                    "320k".to_string()
                },
                ..Default::default()
            },
            output: crate::engine::options::OutputOptions {
                destination_path: expanded_dest.clone(),
                container: format_name.to_lowercase(),
                ..Default::default()
            },
            metadata: crate::engine::options::MetadataOptions {
                embed_metadata: true,
                embed_thumbnail: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let job_config = WizardJobConfig {
            job_id: &job_id,
            url: &target_url,
            title: &info.title,
            mode: mode_str,
            format: format_name,
            quality: quality_name,
            dest_path: &expanded_dest,
            opts,
            engine: &engine,
            db: db.clone(),
        };

        execute_download_job(job_config).await;

        println!();
        let follow_up_options = [
            "Download another video / audio file",
            "Open enclosing folder in Finder / Explorer",
            "View Download History",
            "Exit Siphonix",
        ];

        let act_idx =
            prompt_numbered_choice("What would you like to do next?", &follow_up_options, 0);
        match act_idx {
            0 => continue,
            1 => {
                let _ = opener::open(Path::new(&expanded_dest));
                continue;
            }
            2 => {
                clear_screen();
                print_banner();
                run_history(db.clone()).await;
                let _ = Text::new("  Press Enter to return to main menu...").prompt();
                continue;
            }
            _ => {
                println!(
                    "\n  {}\n",
                    "Thank you for using Siphonix. Goodbye!".cyan().bold()
                );
                break;
            }
        }
    }
}

struct WizardJobConfig<'a> {
    job_id: &'a str,
    url: &'a str,
    title: &'a str,
    mode: &'a str,
    format: &'a str,
    quality: &'a str,
    dest_path: &'a str,
    opts: DownloadOptions,
    engine: &'a EnginePaths,
    db: Arc<DbRepository>,
}

async fn execute_download_job(config: WizardJobConfig<'_>) {
    let request = DownloadRequest {
        url: config.url.to_string(),
        media_mode: config.mode.to_string(),
        audio_format: if config.mode == "audio" {
            Some(config.format.to_string())
        } else {
            None
        },
        audio_quality: if config.mode == "audio" {
            Some(config.quality.to_string())
        } else {
            None
        },
        video_format: if config.mode == "video" {
            Some(config.format.to_string())
        } else {
            None
        },
        video_quality: if config.mode == "video" {
            Some(config.quality.to_string())
        } else {
            None
        },
        destination_path: config.dest_path.to_string(),
        options: Some(config.opts.clone()),
    };

    let (binary, args) = CommandBuilder::build(&request, &config.engine.yt_dlp, Some(&config.engine.ffmpeg));

    let pb = ProgressBar::new(100);
    let pb_style = ProgressStyle::default_bar()
        .template("  {bar:35.cyan/blue} {percent:>3}% | {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("━╸─");
    pb.set_style(pb_style);
    pb.set_message("Initializing download stream...");

    let start_time = Instant::now();

    let mut cmd = Command::new(&binary);
    cmd.args(&args);
    cmd.kill_on_drop(true);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            pb.finish_and_clear();
            eprintln!(
                "\n{} Failed to spawn yt-dlp: {}\n",
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
                "\n{} Failed to capture stdout stream\n",
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
                "\n{} Failed to capture stderr stream\n",
                "Error:".red().bold()
            );
            return;
        }
    };

    let pb_clone = pb.clone();
    let j_id_copy = config.job_id.to_string();
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
            DownloadManager::cleanup_partial_files(Path::new(config.dest_path), "");
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
                    id: config.job_id.to_string(),
                    url: config.url.to_string(),
                    title: config.title.to_string(),
                    thumbnail_url: None,
                    media_mode: config.mode.to_string(),
                    format: config.format.to_string(),
                    quality: config.quality.to_string(),
                    destination_path: config.dest_path.to_string(),
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
                    options: config.opts,
                };

                let _ = config.db.insert_job(&download_job.to_db_job()).await;
                let lib_svc = LibraryService::new(config.db);
                let _ = lib_svc
                    .register_completed_job(&download_job, Some(&path_str))
                    .await;
            }
        }
        Ok(_) => {
            let stderr_text = stderr_buf.join("\n");
            let classified = EngineError::classify_from_stderr(&stderr_text);
            DownloadManager::cleanup_partial_files(Path::new(config.dest_path), "");
            eprintln!(
                "\n{} {}\n",
                "Download Failed:".red().bold(),
                classified.user_friendly_message()
            );
        }
        Err(e) => {
            DownloadManager::cleanup_partial_files(Path::new(config.dest_path), "");
            eprintln!(
                "\n{} Failed to wait on child process: {}\n",
                "Download Failed:".red().bold(),
                e
            );
        }
    }
}
