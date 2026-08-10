import React, { useEffect, useState } from "react";
import {
  Pause,
  Play,
  XCircle,
  RotateCcw,
  CheckCircle2,
  AlertOctagon,
  Clock,
  Inbox,
  ArrowDownCircle,
  Zap,
  ShieldAlert,
} from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { downloadService } from "../services/DownloadService";
import { DownloadState, QueueSummary } from "../types";
import { TauriDownloadService } from "../services/TauriDownloadService";

export const QueueView: React.FC = () => {
  const { jobs, settings } = useAppStore();
  const [summary, setSummary] = useState<QueueSummary | null>(null);

  useEffect(() => {
    if (downloadService instanceof TauriDownloadService) {
      const unsubscribe = downloadService.subscribeSummary(setSummary);
      return () => unsubscribe();
    }
  }, []);

  const activeCount = summary?.active_count ?? jobs.filter(
    (j) => j.state === "DOWNLOADING" || j.state === "PREPARING" || j.state === "PROCESSING"
  ).length;

  const waitingCount = summary?.waiting_count ?? jobs.filter(
    (j) => j.state === "QUEUED" || j.state === "COOLDOWN" || j.state === "RETRYING"
  ).length;

  const completedCount = summary?.completed_count ?? jobs.filter((j) => j.state === "COMPLETED").length;

  const isQueuePaused = summary?.is_paused ?? false;
  const isCooldownActive = summary?.cooldown?.active ?? false;
  const cooldownRemaining = summary?.cooldown?.remaining_secs ?? 0;

  const handleToggleQueuePause = () => {
    if (downloadService instanceof TauriDownloadService) {
      if (isQueuePaused) {
        downloadService.resumeQueue();
      } else {
        downloadService.pauseQueue();
      }
    }
  };

  const handleForceResumeCooldown = () => {
    if (downloadService instanceof TauriDownloadService) {
      downloadService.forceResumeCooldown();
    }
  };

  const formatSecs = (totalSecs: number) => {
    const mins = Math.floor(totalSecs / 60);
    const secs = totalSecs % 60;
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  };

  const getStateBadge = (state: DownloadState) => {
    switch (state) {
      case "DOWNLOADING":
      case "PROCESSING":
      case "PREPARING":
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-blue-500/10 text-blue-600 dark:text-blue-400 text-xs font-semibold flex items-center space-x-1">
            <ArrowDownCircle className="w-3.5 h-3.5 animate-pulse" />
            <span className="capitalize">{state.toLowerCase()}</span>
          </span>
        );
      case "QUEUED":
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-zinc-200 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 text-xs font-medium flex items-center space-x-1">
            <Clock className="w-3.5 h-3.5" />
            <span>Queued</span>
          </span>
        );
      case "RETRYING":
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-amber-500/10 text-amber-600 dark:text-amber-400 text-xs font-medium flex items-center space-x-1">
            <RotateCcw className="w-3.5 h-3.5 animate-spin" />
            <span>Retrying</span>
          </span>
        );
      case "PAUSED":
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-amber-500/10 text-amber-600 dark:text-amber-400 text-xs font-medium flex items-center space-x-1">
            <Pause className="w-3.5 h-3.5" />
            <span>Paused</span>
          </span>
        );
      case "COMPLETED":
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 text-xs font-medium flex items-center space-x-1">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>Completed</span>
          </span>
        );
      case "FAILED":
      case "NEEDS_ATTENTION":
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-red-500/10 text-red-600 dark:text-red-400 text-xs font-medium flex items-center space-x-1">
            <AlertOctagon className="w-3.5 h-3.5" />
            <span className="capitalize">{state.toLowerCase()}</span>
          </span>
        );
      case "CANCELLED":
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-zinc-200 dark:bg-zinc-800 text-zinc-500 text-xs font-medium">
            Cancelled
          </span>
        );
      default:
        return (
          <span className="px-2.5 py-0.5 rounded-full bg-zinc-200 dark:bg-zinc-800 text-zinc-500 text-xs font-medium">
            {state}
          </span>
        );
    }
  };

  return (
    <div className="max-w-4xl mx-auto py-8 px-6 space-y-6">
      {/* Header & Controls */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-surface-border pb-4">
        <div>
          <h1 className="text-xl font-bold text-zinc-900 dark:text-zinc-100 tracking-tight">
            Download Queue
          </h1>
          <p className="text-xs text-zinc-500 dark:text-zinc-400">
            Multi-download scheduler with automatic reliability & backoff.
          </p>
        </div>

        <div className="flex items-center space-x-3 text-xs font-mono">
          <div className="px-3 py-1.5 rounded-lg bg-surface-card border border-surface-border flex space-x-3">
            <div>
              <span className="text-zinc-500">Active: </span>
              <span className="font-semibold text-brand-600 dark:text-brand-400">
                {activeCount} / {settings.maxConcurrentDownloads}
              </span>
            </div>
            <div>
              <span className="text-zinc-500">Waiting: </span>
              <span className="font-semibold text-zinc-800 dark:text-zinc-200">
                {waitingCount}
              </span>
            </div>
            <div>
              <span className="text-zinc-500">Done: </span>
              <span className="font-semibold text-emerald-500">
                {completedCount}
              </span>
            </div>
          </div>

          <button
            onClick={handleToggleQueuePause}
            className={`px-3 py-1.5 rounded-lg border text-xs font-medium font-sans flex items-center space-x-1.5 transition-colors ${
              isQueuePaused
                ? "bg-emerald-500/10 border-emerald-500 text-emerald-600 dark:text-emerald-400"
                : "bg-zinc-100 dark:bg-zinc-800 border-surface-border text-zinc-700 dark:text-zinc-300 hover:bg-surface-hover"
            }`}
          >
            {isQueuePaused ? <Play className="w-3.5 h-3.5" /> : <Pause className="w-3.5 h-3.5" />}
            <span>{isQueuePaused ? "Resume Queue" : "Pause Queue"}</span>
          </button>
        </div>
      </div>

      {/* Queue-Wide Cooldown Banner */}
      {isCooldownActive && (
        <div className="bg-amber-500/10 border border-amber-500/30 rounded-xl p-4 flex items-center justify-between shadow-subtle">
          <div className="flex items-center space-x-3">
            <div className="w-9 h-9 rounded-lg bg-amber-500/20 text-amber-500 flex items-center justify-center">
              <ShieldAlert className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-amber-600 dark:text-amber-400">
                Taking a short break
              </h3>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                YouTube isn't responding normally right now. Siphonix will automatically resume in{" "}
                <span className="font-mono font-bold text-amber-500">{formatSecs(cooldownRemaining)}</span>.
              </p>
            </div>
          </div>

          <button
            onClick={handleForceResumeCooldown}
            className="px-3.5 py-1.5 bg-amber-500 hover:bg-amber-600 text-white rounded-lg text-xs font-semibold flex items-center space-x-1.5 transition-colors whitespace-nowrap shadow-subtle"
          >
            <Zap className="w-3.5 h-3.5" />
            <span>Resume Now</span>
          </button>
        </div>
      )}

      {/* Queue List */}
      {jobs.length === 0 ? (
        <div className="bg-surface-card rounded-xl border border-surface-border p-12 text-center space-y-3">
          <div className="w-12 h-12 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center mx-auto text-zinc-400">
            <Inbox className="w-6 h-6" />
          </div>
          <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
            Your download queue is empty
          </h3>
          <p className="text-xs text-zinc-500 max-w-sm mx-auto">
            Paste a YouTube video or playlist link in the Download tab to start downloading.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {jobs.map((job) => (
            <div
              key={job.id}
              className="bg-surface-card rounded-xl border border-surface-border p-4 shadow-subtle space-y-3 transition-all"
            >
              <div className="flex items-start justify-between">
                <div className="space-y-1">
                  <div className="flex items-center space-x-2">
                    <h3 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                      {job.title}
                    </h3>
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400">
                      {job.format} ({job.quality})
                    </span>
                  </div>
                  <p className="text-xs font-mono text-zinc-400 truncate max-w-lg">
                    {job.url}
                  </p>
                </div>

                <div className="flex items-center space-x-2">
                  {getStateBadge(job.state)}

                  {(job.state === "FAILED" || job.state === "CANCELLED") && (
                    <button
                      onClick={() => downloadService.retryJob(job.id)}
                      title="Retry"
                      className="p-1.5 text-zinc-400 hover:text-brand-500 transition-colors"
                    >
                      <RotateCcw className="w-4 h-4" />
                    </button>
                  )}

                  {job.state !== "COMPLETED" && (
                    <button
                      onClick={() => downloadService.cancelJob(job.id)}
                      title="Cancel"
                      className="p-1.5 text-zinc-400 hover:text-red-500 transition-colors"
                    >
                      <XCircle className="w-4 h-4" />
                    </button>
                  )}
                </div>
              </div>

              {/* Retry Countdown Banner */}
              {job.state === "RETRYING" && (
                <div className="text-xs font-mono text-amber-500 bg-amber-500/10 px-3 py-1.5 rounded-md flex items-center justify-between">
                  <span>Temporary issue detected. Retrying automatically...</span>
                  <span className="font-semibold">Attempt {job.retryCount || 1} / {job.maxRetries || 5}</span>
                </div>
              )}

              {/* Failed Error Details */}
              {job.state === "FAILED" && job.errorMessage && (
                <div className="text-xs font-mono text-red-500 bg-red-500/10 px-3 py-1.5 rounded-md">
                  <span>{job.errorMessage}</span>
                </div>
              )}

              {/* Progress Bar & Telemetry */}
              {job.state !== "COMPLETED" && job.state !== "CANCELLED" && job.state !== "FAILED" && (
                <div className="space-y-1.5">
                  <div className="h-1.5 w-full bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-brand-500 transition-all duration-300 rounded-full"
                      style={{ width: `${job.progress}%` }}
                    />
                  </div>

                  <div className="flex justify-between text-[11px] font-mono text-zinc-400">
                    <span>{job.progress}% completed</span>
                    {job.downloadSpeed && <span>{job.downloadSpeed}</span>}
                    {job.eta && <span>ETA: {job.eta}</span>}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
