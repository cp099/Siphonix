import { z } from "zod";

export type DownloadState =
  | "QUEUED"
  | "PREPARING"
  | "DOWNLOADING"
  | "PROCESSING"
  | "PAUSED"
  | "RETRYING"
  | "COOLDOWN"
  | "COMPLETED"
  | "FAILED"
  | "CANCELLED"
  | "NEEDS_ATTENTION";

export type MediaMode = "audio" | "video";
export type AudioFormat = "MP3" | "M4A" | "AAC" | "FLAC" | "ALAC" | "OPUS" | "WAV";
export type AudioQuality = "best" | "320k" | "256k" | "192k" | "128k";
export type VideoFormat = "MP4" | "MKV" | "WEBM";
export type VideoQuality = "best" | "2160p" | "1440p" | "1080p" | "720p" | "480p" | "360p";

export type UrlType = "VIDEO" | "PLAYLIST" | "VIDEO_WITH_PLAYLIST" | "INVALID";
export type FileStatus = "AVAILABLE" | "MISSING" | "INACCESSIBLE" | "UNKNOWN";

export interface LibraryItem {
  id: string;
  jobId: string;
  sourceVideoId?: string;
  title: string;
  filePath: string;
  fileName: string;
  fileExtension: string;
  mediaMode: MediaMode;
  format: string;
  quality: string;
  fileSizeBytes: number;
  durationSeconds?: number;
  thumbnailUrl?: string;
  sourceUrl: string;
  sourcePlaylistId?: string;
  sourcePlaylistTitle?: string;
  playlistEntryIndex?: number;
  createdAt: string;
  completedAt: string;
  lastVerifiedAt?: string;
  fileStatus: "AVAILABLE" | "MISSING";
  optionsJson?: string;
}

export type EngineSource = "Managed" | "System" | "DevOverride" | "None";
export type EngineStatusState = "READY" | "MISSING" | "OUTDATED" | "INCOMPATIBLE" | "CORRUPTED" | "CHECKING";

export interface EngineInfo {
  name: string;
  path?: string;
  version?: string;
  source: EngineSource;
  compatible: boolean;
  status: EngineStatusState;
  error?: string;
}

export interface Diagnostic {
  code: string;
  level: "info" | "warning" | "error";
  message: string;
}

export interface RuntimeStatus {
  ready: boolean;
  yt_dlp: EngineInfo;
  ffmpeg: EngineInfo;
  diagnostics: Diagnostic[];
}

export type DiagnosticSeverity = "DEBUG" | "INFO" | "WARN" | "ERROR" | "CRITICAL";

export interface DiagnosticEvent {
  id: string;
  timestamp: string;
  severity: DiagnosticSeverity;
  subsystem: string;
  event_type: string;
  job_id?: string;
  engine_info?: string;
  message: string;
  context?: Record<string, any>;
}

export type SystemHealthStatus = "HEALTHY" | "DEGRADED" | "ACTION_REQUIRED";

export interface SubsystemHealth {
  name: string;
  status: SystemHealthStatus;
  message: string;
}

export interface SystemHealth {
  overall_status: SystemHealthStatus;
  runtime: SubsystemHealth;
  database: SubsystemHealth;
  queue: SubsystemHealth;
  library: SubsystemHealth;
  active_issues_count: number;
}

export interface DiagnosticReport {
  app_name: string;
  app_version: string;
  platform: string;
  architecture: string;
  generated_at: string;
  system_health: SystemHealth;
  runtime_status: RuntimeStatus;
  total_jobs_count: number;
  total_library_items_count: number;
  recent_events: DiagnosticEvent[];
}

export interface RecoveryResult {
  subsystem: string;
  success: boolean;
  message: string;
  items_affected: number;
}

export const UrlInputSchema = z.string().trim().min(1, "URL cannot be empty").refine(
  (val) => val.includes("youtube.com/") || val.includes("youtu.be/"),
  { message: "Please enter a valid YouTube video or playlist link" }
);

export interface UrlValidationResult {
  valid: boolean;
  url_type: UrlType;
  is_playlist: boolean;
  video_id: string | null;
  playlist_id: string | null;
  message: string;
}

export interface AppInfo {
  name: string;
  version: string;
  platform: string;
}

export interface VideoOptions {
  resolution: VideoQuality;
  frame_rate: string;
  codec_preference: string;
  hdr_preference: string;
  selection_mode: "prefer" | "require";
}

export interface AudioOptions {
  format: AudioFormat;
  quality: AudioQuality;
  codec_preference: string;
}

export interface OutputOptions {
  container: string;
  destination_path: string;
  naming_preset: string;
  custom_naming_template?: string;
  folder_organization: string;
  overwrite_policy: string;
}

export interface MetadataOptions {
  embed_metadata: boolean;
  embed_thumbnail: boolean;
  write_metadata_json: boolean;
}

export interface SubtitleOptions {
  enabled: boolean;
  languages: string[];
  format: string;
  embed_in_video: boolean;
}

export interface NetworkOptions {
  concurrent_fragments: string;
  max_download_rate?: string;
}

export interface ExpertOptions {
  format_sort_strategy: string;
}

export interface DownloadOptions {
  media_mode: MediaMode;
  video: VideoOptions;
  audio: AudioOptions;
  output: OutputOptions;
  metadata: MetadataOptions;
  subtitles: SubtitleOptions;
  network: NetworkOptions;
  expert: ExpertOptions;
}

export interface DownloadPreset {
  id: string;
  name: string;
  description?: string;
  is_default: boolean;
  options: DownloadOptions;
  created_at: string;
  updated_at: string;
}

export interface CreateJobParams {
  url: string;
  title?: string;
  mediaMode: MediaMode;
  audioFormat?: AudioFormat;
  audioQuality?: AudioQuality;
  videoFormat?: VideoFormat;
  videoQuality?: VideoQuality;
  destinationPath: string;
  isPlaylist?: boolean;
  options?: DownloadOptions;
}

export interface DownloadJob {
  id: string;
  url: string;
  title: string;
  thumbnailUrl?: string;
  mediaMode: MediaMode;
  format: string;
  quality: string;
  destinationPath: string;
  state: DownloadState;
  progress: number;
  downloadSpeed?: string;
  eta?: string;
  fileSize?: string;
  errorMessage?: string;
  lastErrorCategory?: string;
  retryCount?: number;
  maxRetries?: number;
  nextRetryAt?: string;
  createdAt: string;
  completedAt?: string;
  sourceVideoId?: string;
  sourcePlaylistId?: string;
  sourcePlaylistTitle?: string;
  playlistEntryIndex?: number;
  options?: DownloadOptions;
}

export interface PlaylistEntry {
  id: string;
  index: number;
  url: string;
  title: string;
  duration?: number;
  thumbnail_url?: string;
  availability: string; // "AVAILABLE" | "UNAVAILABLE" | "PRIVATE" | "DELETED" | "UNKNOWN"
}

export interface PlaylistInfo {
  id: string;
  title: string;
  uploader?: string;
  webpage_url?: string;
  entry_count: number;
  available_count: number;
  entries: PlaylistEntry[];
}

export interface EnqueuePlaylistParams {
  playlist_id: string;
  playlist_title: string;
  entries: PlaylistEntry[];
  media_mode: MediaMode;
  audio_format?: AudioFormat;
  audio_quality?: AudioQuality;
  video_format?: VideoFormat;
  video_quality?: VideoQuality;
  destination_path: string;
  options?: DownloadOptions;
}

export interface EnqueuePlaylistResult {
  added_count: number;
  skipped_count: number;
}

export interface CooldownStatus {
  active: boolean;
  remaining_secs: number;
}

export interface QueueSummary {
  active_count: number;
  waiting_count: number;
  completed_count: number;
  failed_count: number;
  max_concurrency: number;
  is_paused: boolean;
  cooldown: CooldownStatus;
}

export interface UserSettings {
  theme: "dark" | "light" | "system";
  defaultDestination: string;
  maxConcurrentDownloads: number;
  autoRetry: boolean;
  maxRetries: number;
}
