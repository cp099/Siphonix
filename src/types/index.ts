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

export const UrlInputSchema = z.string().trim().min(1, "URL cannot be empty").refine(
  (val) => val.includes("youtube.com/") || val.includes("youtu.be/"),
  { message: "Please enter a valid YouTube video or playlist link" }
);

export interface UrlValidationResult {
  valid: boolean;
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
