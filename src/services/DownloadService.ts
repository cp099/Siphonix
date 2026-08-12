import {
  AppInfo,
  CreateJobParams,
  DownloadJob,
  UrlValidationResult,
} from "../types";
import { invokeGetAppInfo, invokeValidateUrl, isTauriEnv, pickFolder } from "../lib/tauriBridge";
import { TauriDownloadService } from "./TauriDownloadService";

export interface IDownloadService {
  validateUrl(url: string): Promise<UrlValidationResult>;
  getAppInfo(): Promise<AppInfo>;
  pickDestinationFolder(defaultPath?: string): Promise<string | null>;
  enqueueJob(params: CreateJobParams): Promise<DownloadJob>;
  getQueue(): Promise<DownloadJob[]>;
  pauseJob(id: string): Promise<void>;
  resumeJob(id: string): Promise<void>;
  cancelJob(id: string): Promise<void>;
  retryJob(id: string): Promise<void>;
  getRuntimeStatus?(): Promise<any>;
  refreshRuntimeStatus?(): Promise<any>;
  getDiagnostics?(limit?: number): Promise<any>;
  getSystemHealth?(): Promise<any>;
  generateDiagnosticReport?(): Promise<any>;
  verifyDatabase?(): Promise<any>;
  verifyLibrary?(): Promise<any>;
  recoverInterruptedJobs?(): Promise<any>;
}

/**
 * Phase 1 Mock Download Service implementation.
 * Used as web dev fallback when running outside of Tauri.
 */
export class MockDownloadService implements IDownloadService {
  private queue: DownloadJob[] = [];
  private listeners: Set<(jobs: DownloadJob[]) => void> = new Set();

  async validateUrl(url: string): Promise<UrlValidationResult> {
    return invokeValidateUrl(url);
  }

  async getAppInfo(): Promise<AppInfo> {
    return invokeGetAppInfo();
  }

  async pickDestinationFolder(defaultPath?: string): Promise<string | null> {
    return pickFolder(defaultPath);
  }

  async enqueueJob(params: CreateJobParams): Promise<DownloadJob> {
    const isAudio = params.mediaMode === "audio";
    const formatStr = isAudio ? params.audioFormat || "MP3" : params.videoFormat || "MP4";
    const qualityStr = isAudio ? params.audioQuality || "best" : params.videoQuality || "1080p";

    const newJob: DownloadJob = {
      id: `job-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
      url: params.url,
      title: params.title || (params.isPlaylist ? "YouTube Playlist Download" : "YouTube Video Download"),
      thumbnailUrl: undefined,
      mediaMode: params.mediaMode,
      format: formatStr,
      quality: qualityStr,
      destinationPath: params.destinationPath,
      state: "QUEUED",
      progress: 0,
      createdAt: new Date().toISOString(),
    };

    this.queue.unshift(newJob);
    this.notifyListeners();

    // Simulate mock progress transition for web browser dev fallback
    this.simulateMockProgress(newJob.id);

    return newJob;
  }

  async getQueue(): Promise<DownloadJob[]> {
    return [...this.queue];
  }

  async pauseJob(id: string): Promise<void> {
    const job = this.queue.find((j) => j.id === id);
    if (job && (job.state === "DOWNLOADING" || job.state === "QUEUED")) {
      job.state = "PAUSED";
      this.notifyListeners();
    }
  }

  async resumeJob(id: string): Promise<void> {
    const job = this.queue.find((j) => j.id === id);
    if (job && job.state === "PAUSED") {
      job.state = "DOWNLOADING";
      this.notifyListeners();
      this.simulateMockProgress(id);
    }
  }

  async cancelJob(id: string): Promise<void> {
    const job = this.queue.find((j) => j.id === id);
    if (job) {
      job.state = "CANCELLED";
      this.notifyListeners();
    }
  }

  async retryJob(id: string): Promise<void> {
    const job = this.queue.find((j) => j.id === id);
    if (job) {
      job.state = "QUEUED";
      job.progress = 0;
      this.notifyListeners();
      this.simulateMockProgress(id);
    }
  }

  public subscribe(listener: (jobs: DownloadJob[]) => void): () => void {
    this.listeners.add(listener);
    listener(this.queue);
    return () => this.listeners.delete(listener);
  }

  private notifyListeners() {
    for (const listener of this.listeners) {
      listener([...this.queue]);
    }
  }

  private simulateMockProgress(jobId: string) {
    let currentProgress = 0;
    const interval = setInterval(() => {
      const job = this.queue.find((j) => j.id === jobId);
      if (!job || job.state === "PAUSED" || job.state === "CANCELLED" || job.state === "COMPLETED") {
        clearInterval(interval);
        return;
      }

      if (job.state === "QUEUED") {
        job.state = "DOWNLOADING";
        job.downloadSpeed = "5.4 MB/s";
        job.fileSize = "34.2 MB";
      }

      currentProgress += 15;
      if (currentProgress >= 100) {
        job.progress = 100;
        job.state = "COMPLETED";
        job.completedAt = new Date().toISOString();
        job.downloadSpeed = undefined;
        job.eta = undefined;
        clearInterval(interval);
      } else {
        job.progress = currentProgress;
        job.eta = `00:${Math.max(1, Math.ceil((100 - currentProgress) / 15))}s`;
      }

      this.notifyListeners();
    }, 800);
  }
}

// Export active download service based on execution environment
export const downloadService: IDownloadService = isTauriEnv()
  ? new TauriDownloadService()
  : new MockDownloadService();
