import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  AppInfo,
  CreateJobParams,
  DownloadJob,
  EnqueuePlaylistParams,
  EnqueuePlaylistResult,
  PlaylistInfo,
  QueueSummary,
  UrlValidationResult,
} from "../types";
import { IDownloadService } from "./DownloadService";
import { invokeGetAppInfo, invokeValidateUrl, pickFolder } from "../lib/tauriBridge";

export class TauriDownloadService implements IDownloadService {
  private jobs: DownloadJob[] = [];
  private listeners: Set<(jobs: DownloadJob[]) => void> = new Set();
  private summaryListeners: Set<(summary: QueueSummary) => void> = new Set();

  constructor() {
    this.setupEventListeners();
  }

  private async setupEventListeners() {
    try {
      await listen<DownloadJob[]>("jobs-refreshed", (event) => {
        this.jobs = event.payload;
        this.notifyListeners();
      });

      await listen<QueueSummary>("queue-updated", (event) => {
        for (const listener of this.summaryListeners) {
          listener(event.payload);
        }
      });

      this.refreshQueue();
    } catch (err) {
      console.warn("Failed to attach Tauri event listeners:", err);
    }
  }

  async refreshQueue(): Promise<void> {
    try {
      const jobs = await invoke<DownloadJob[]>("get_queue_jobs");
      this.jobs = jobs;
      this.notifyListeners();
    } catch {
      // Fallback
    }
  }

  async validateUrl(url: string): Promise<UrlValidationResult> {
    return invokeValidateUrl(url);
  }

  async getAppInfo(): Promise<AppInfo> {
    return invokeGetAppInfo();
  }

  async pickDestinationFolder(defaultPath?: string): Promise<string | null> {
    return pickFolder(defaultPath);
  }

  async inspectPlaylist(url: string, inspectionId: string = "insp-default"): Promise<PlaylistInfo> {
    return invoke<PlaylistInfo>("inspect_playlist_url", { inspectionId, url });
  }

  async cancelPlaylistInspection(inspectionId: string = "insp-default"): Promise<boolean> {
    return invoke<boolean>("cancel_playlist_inspection", { inspectionId });
  }

  async enqueuePlaylist(params: EnqueuePlaylistParams): Promise<EnqueuePlaylistResult> {
    const res = await invoke<EnqueuePlaylistResult>("enqueue_playlist_entries", { params });
    await this.refreshQueue();
    return res;
  }

  async enqueueJob(params: CreateJobParams): Promise<DownloadJob> {
    const job = await invoke<DownloadJob>("enqueue_download", {
      request: {
        url: params.url,
        media_mode: params.mediaMode,
        audio_format: params.audioFormat,
        audio_quality: params.audioQuality,
        video_format: params.videoFormat,
        video_quality: params.videoQuality,
        destination_path: params.destinationPath,
      },
    });

    await this.refreshQueue();
    return job;
  }

  async getQueue(): Promise<DownloadJob[]> {
    return invoke<DownloadJob[]>("get_queue_jobs");
  }

  async getLibraryJobs(): Promise<DownloadJob[]> {
    return invoke<DownloadJob[]>("get_library_jobs");
  }

  async pauseJob(id: string): Promise<void> {
    await invoke("cancel_job", { jobId: id });
    await this.refreshQueue();
  }

  async resumeJob(_id: string): Promise<void> {
    await this.refreshQueue();
  }

  async cancelJob(id: string): Promise<void> {
    await invoke("cancel_job", { jobId: id });
    await this.refreshQueue();
  }

  async retryJob(id: string): Promise<void> {
    const job = this.jobs.find((j) => j.id === id);
    if (job) {
      await this.enqueueJob({
        url: job.url,
        title: job.title,
        mediaMode: job.mediaMode,
        destinationPath: job.destinationPath,
      });
    }
  }

  async pauseQueue(): Promise<void> {
    await invoke("pause_queue");
  }

  async resumeQueue(): Promise<void> {
    await invoke("resume_queue");
  }

  async forceResumeCooldown(): Promise<void> {
    await invoke("force_resume_cooldown");
  }

  async setMaxConcurrency(limit: number): Promise<void> {
    await invoke("set_max_concurrency", { limit });
  }

  public subscribe(listener: (jobs: DownloadJob[]) => void): () => void {
    this.listeners.add(listener);
    listener([...this.jobs]);
    return () => this.listeners.delete(listener);
  }

  public subscribeSummary(listener: (summary: QueueSummary) => void): () => void {
    this.summaryListeners.add(listener);
    return () => this.summaryListeners.delete(listener);
  }

  private notifyListeners() {
    for (const listener of this.listeners) {
      listener([...this.jobs]);
    }
  }
}
