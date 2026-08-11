import React, { useState } from "react";
import {
  Download,
  Folder,
  Music,
  Video,
  ListMusic,
  CheckCircle2,
  AlertCircle,
  Loader2,
  CheckSquare,
  Square,
} from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { downloadService } from "../services/DownloadService";
import { TauriDownloadService } from "../services/TauriDownloadService";
import {
  AudioFormat,
  AudioQuality,
  DownloadOptions,
  MediaMode,
  PlaylistInfo,
  VideoFormat,
  VideoQuality,
} from "../types";
import { AdvancedOptionsDrawer } from "./AdvancedOptionsDrawer";

export const DownloadView: React.FC = () => {
  const { url, setUrl, settings } = useAppStore();
  const [inputUrl, setInputUrl] = useState(url);
  const [mediaMode, setMediaMode] = useState<MediaMode>("video");
  const [audioFormat, setAudioFormat] = useState<AudioFormat>("MP3");
  const [audioQuality, setAudioQuality] = useState<AudioQuality>("best");
  const [videoFormat, setVideoFormat] = useState<VideoFormat>("MP4");
  const [videoQuality, setVideoQuality] = useState<VideoQuality>("1080p");
  const [destinationPath, setDestinationPath] = useState(settings.defaultDestination);

  const [showAdvancedDrawer, setShowAdvancedDrawer] = useState(false);
  const [downloadOptions, setDownloadOptions] = useState<DownloadOptions>({
    media_mode: "video",
    video: {
      resolution: "1080p",
      frame_rate: "auto",
      codec_preference: "auto",
      hdr_preference: "auto",
      selection_mode: "prefer",
    },
    audio: {
      format: "MP3",
      quality: "best",
      codec_preference: "auto",
    },
    output: {
      container: "MP4",
      destination_path: settings.defaultDestination,
      naming_preset: "simple",
      folder_organization: "flat",
      overwrite_policy: "ask",
    },
    metadata: {
      embed_metadata: true,
      embed_thumbnail: true,
      write_metadata_json: false,
    },
    subtitles: {
      enabled: false,
      languages: ["en"],
      format: "srt",
      embed_in_video: false,
    },
    network: {
      concurrent_fragments: "auto",
    },
    expert: {
      format_sort_strategy: "resolution_first",
    },
  });

  const [isValidating, setIsValidating] = useState(false);
  const [validationResult, setValidationResult] = useState<{
    valid: boolean;
    url_type?: "VIDEO" | "PLAYLIST" | "VIDEO_WITH_PLAYLIST" | "INVALID";
    is_playlist?: boolean;
    video_id?: string | null;
    playlist_id?: string | null;
    message?: string;
  } | null>(null);

  // Playlist Inspection & Selection state
  const [isInspecting, setIsInspecting] = useState(false);
  const [playlistInfo, setPlaylistInfo] = useState<PlaylistInfo | null>(null);
  const [selectedEntryIds, setSelectedEntryIds] = useState<Set<String>>(new Set());
  const [inspectionError, setInspectionError] = useState<string | null>(null);

  // Feedback status
  const [feedbackMsg, setFeedbackMsg] = useState<{ type: "success" | "error"; text: string } | null>(null);
  const [runtimeStatus, setRuntimeStatus] = useState<any>(null);

  React.useEffect(() => {
    if (downloadService.getRuntimeStatus) {
      downloadService.getRuntimeStatus().then(setRuntimeStatus).catch(() => {});
    }
  }, []);

  const handleUrlChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setInputUrl(val);
    setUrl(val);
    setPlaylistInfo(null);
    setInspectionError(null);
    setFeedbackMsg(null);

    if (val.trim().length > 10) {
      setIsValidating(true);
      try {
        const res = await downloadService.validateUrl(val);
        setValidationResult(res);

        // Pure PLAYLIST URL -> trigger inspection automatically
        if (res.valid && res.url_type === "PLAYLIST") {
          handleInspectPlaylist(val);
        }
      } catch {
        setValidationResult({ valid: false, message: "Validation error" });
      } finally {
        setIsValidating(false);
      }
    } else {
      setValidationResult(null);
    }
  };

  const handlePickFolder = async () => {
    const picked = await downloadService.pickDestinationFolder(destinationPath);
    if (picked) {
      setDestinationPath(picked);
    }
  };

  const handleInspectPlaylist = async (urlToInspect?: string) => {
    const targetUrl = urlToInspect || inputUrl;
    setIsInspecting(true);
    setInspectionError(null);
    setFeedbackMsg(null);

    try {
      if (downloadService instanceof TauriDownloadService) {
        const info = await downloadService.inspectPlaylist(targetUrl, "insp-active");
        setPlaylistInfo(info);

        // Pre-select all available entries
        const availableIds = new Set(
          info.entries.filter((e) => e.availability === "AVAILABLE").map((e) => e.id)
        );
        setSelectedEntryIds(availableIds);
      }
    } catch (err) {
      setInspectionError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsInspecting(false);
    }
  };

  const handleCancelInspection = async () => {
    if (downloadService instanceof TauriDownloadService) {
      await downloadService.cancelPlaylistInspection("insp-active");
      setIsInspecting(false);
    }
  };

  // Selection handlers
  const toggleSelectEntry = (id: string, isAvailable: boolean) => {
    if (!isAvailable) return;
    const next = new Set(selectedEntryIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setSelectedEntryIds(next);
  };

  const handleSelectAll = () => {
    if (!playlistInfo) return;
    const availableIds = new Set(
      playlistInfo.entries.filter((e) => e.availability === "AVAILABLE").map((e) => e.id)
    );
    setSelectedEntryIds(availableIds);
  };

  const handleDeselectAll = () => {
    setSelectedEntryIds(new Set());
  };

  const handleSelectAvailableOnly = () => {
    handleSelectAll();
  };

  // Submit Single Download
  const handleStartSingleDownload = async () => {
    try {
      const effectiveOpts: DownloadOptions = {
        ...downloadOptions,
        media_mode: mediaMode,
        video: { ...downloadOptions.video, resolution: videoQuality },
        audio: { ...downloadOptions.audio, format: audioFormat, quality: audioQuality },
        output: { ...downloadOptions.output, container: videoFormat, destination_path: destinationPath },
      };

      await downloadService.enqueueJob({
        url: inputUrl,
        mediaMode,
        audioFormat,
        audioQuality,
        videoFormat,
        videoQuality,
        destinationPath,
        options: effectiveOpts,
      });
      setFeedbackMsg({ type: "success", text: "Download added to queue with advanced options snapshot!" });
    } catch (err) {
      setFeedbackMsg({ type: "error", text: err instanceof Error ? err.message : String(err) });
    }
  };

  // Submit Playlist Queue Expansion
  const handleEnqueuePlaylist = async () => {
    if (!playlistInfo || selectedEntryIds.size === 0) return;

    const selectedEntries = playlistInfo.entries.filter((e) => selectedEntryIds.has(e.id));

    try {
      const effectiveOpts: DownloadOptions = {
        ...downloadOptions,
        media_mode: mediaMode,
        video: { ...downloadOptions.video, resolution: videoQuality },
        audio: { ...downloadOptions.audio, format: audioFormat, quality: audioQuality },
        output: { ...downloadOptions.output, container: videoFormat, destination_path: destinationPath },
      };

      if (downloadService instanceof TauriDownloadService) {
        const res = await downloadService.enqueuePlaylist({
          playlist_id: playlistInfo.id,
          playlist_title: playlistInfo.title,
          entries: selectedEntries,
          media_mode: mediaMode,
          audio_format: audioFormat,
          audio_quality: audioQuality,
          video_format: videoFormat,
          video_quality: videoQuality,
          destination_path: destinationPath,
          options: effectiveOpts,
        });

        let msg = `${res.added_count} items added to queue.`;
        if (res.skipped_count > 0) {
          msg += ` (${res.skipped_count} existing duplicates skipped)`;
        }
        setFeedbackMsg({ type: "success", text: msg });
      }
    } catch (err) {
      setFeedbackMsg({ type: "error", text: err instanceof Error ? err.message : String(err) });
    }
  };

  return (
    <div className="max-w-4xl mx-auto py-8 px-6 space-y-8">
      {/* Title */}
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold text-zinc-900 dark:text-zinc-100 tracking-tight">
          Siphonix Download Manager
        </h1>
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          Paste any YouTube video or playlist link to inspect and queue downloads.
        </p>
      </div>

      {/* First-Run Readiness Banner */}
      {runtimeStatus && !runtimeStatus.ready && (
        <div className="p-4 rounded-xl border border-red-500/30 bg-red-500/10 text-red-400 text-xs flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <AlertCircle className="w-5 h-5 shrink-0 text-red-500" />
            <div>
              <p className="font-semibold text-white">Siphonix isn't ready yet</p>
              <p className="text-zinc-300">The download engine is unavailable or incompatible. Check Settings → Runtime for diagnostics.</p>
            </div>
          </div>
          <button
            onClick={() => {
              if (downloadService.refreshRuntimeStatus) {
                downloadService.refreshRuntimeStatus().then(setRuntimeStatus);
              }
            }}
            className="px-3 py-1 bg-red-500 hover:bg-red-600 text-white rounded font-medium transition-colors"
          >
            Check Again
          </button>
        </div>
      )}

      {/* URL Input Bar */}
      <div className="bg-surface-card rounded-2xl border border-surface-border p-4 shadow-subtle space-y-4">
        <div className="relative flex items-center">
          <input
            type="text"
            value={inputUrl}
            onChange={handleUrlChange}
            placeholder="Paste YouTube Video or Playlist URL (e.g. https://www.youtube.com/watch?v=...)"
            className="w-full px-4 py-3.5 bg-surface hover:bg-surface-hover rounded-xl border border-surface-border focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20 outline-none font-mono text-sm text-zinc-900 dark:text-zinc-100 transition-all pr-24"
          />

          <div className="absolute right-3 flex items-center space-x-2">
            {isValidating && <Loader2 className="w-5 h-5 text-brand-500 animate-spin" />}
            {validationResult?.valid && <CheckCircle2 className="w-5 h-5 text-emerald-500" />}
            {validationResult && !validationResult.valid && <AlertCircle className="w-5 h-5 text-red-500" />}
          </div>
        </div>

        {/* Validation Status / Secondary Playlist Trigger */}
        {validationResult && (
          <div className="flex items-center justify-between text-xs font-mono px-2">
            <span
              className={
                validationResult.valid
                  ? "text-emerald-600 dark:text-emerald-400"
                  : "text-red-500"
              }
            >
              {validationResult.message}
            </span>

            {/* VIDEO_WITH_PLAYLIST: Secondary Inspect Playlist trigger */}
            {validationResult.url_type === "VIDEO_WITH_PLAYLIST" && !playlistInfo && (
              <button
                onClick={() => handleInspectPlaylist()}
                disabled={isInspecting}
                className="text-brand-600 dark:text-brand-400 hover:underline flex items-center space-x-1 font-sans font-medium"
              >
                <ListMusic className="w-3.5 h-3.5" />
                <span>Inspect Full Playlist ({validationResult.playlist_id})</span>
              </button>
            )}
          </div>
        )}
      </div>

      {/* Feedback Alert */}
      {inspectionError && (
        <div className="p-4 rounded-xl border bg-red-500/10 border-red-500/30 text-red-600 dark:text-red-400 text-sm font-medium flex items-center space-x-2">
          <AlertCircle className="w-5 h-5" />
          <span>Playlist Inspection Error: {inspectionError}</span>
        </div>
      )}

      {feedbackMsg && (
        <div
          className={`p-4 rounded-xl border text-sm font-medium flex items-center space-x-2 ${
            feedbackMsg.type === "success"
              ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-600 dark:text-emerald-400"
              : "bg-red-500/10 border-red-500/30 text-red-600 dark:text-red-400"
          }`}
        >
          {feedbackMsg.type === "success" ? (
            <CheckCircle2 className="w-5 h-5" />
          ) : (
            <AlertCircle className="w-5 h-5" />
          )}
          <span>{feedbackMsg.text}</span>
        </div>
      )}

      {/* Playlist Inspection Spinner */}
      {isInspecting && (
        <div className="bg-surface-card rounded-2xl border border-surface-border p-8 text-center space-y-4 shadow-subtle">
          <Loader2 className="w-8 h-8 text-brand-500 animate-spin mx-auto" />
          <div>
            <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
              Inspecting Playlist Metadata...
            </h3>
            <p className="text-xs text-zinc-500">
              Enumerating playlist entries efficiently via lightweight yt-dlp inspection.
            </p>
          </div>
          <button
            onClick={handleCancelInspection}
            className="px-3.5 py-1.5 rounded-lg border border-red-500/30 text-red-500 text-xs font-medium hover:bg-red-500/10 transition-colors"
          >
            Cancel Inspection
          </button>
        </div>
      )}

      {/* Playlist Inspection Card */}
      {playlistInfo && (
        <div className="bg-surface-card rounded-2xl border border-surface-border p-6 shadow-subtle space-y-6">
          {/* Header */}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between border-b border-surface-border pb-4 gap-4">
            <div>
              <div className="flex items-center space-x-2">
                <h2 className="text-lg font-bold text-zinc-900 dark:text-zinc-100">
                  {playlistInfo.title}
                </h2>
                <span className="px-2.5 py-0.5 rounded-full bg-brand-500/10 text-brand-600 dark:text-brand-400 text-xs font-mono font-semibold">
                  Playlist
                </span>
              </div>
              {playlistInfo.uploader && (
                <p className="text-xs text-zinc-500 dark:text-zinc-400">
                  by {playlistInfo.uploader} • {playlistInfo.entry_count} videos ({playlistInfo.available_count} available)
                </p>
              )}
            </div>

            {/* Selection Toolbar */}
            <div className="flex items-center space-x-2 text-xs">
              <span className="font-mono text-zinc-400 mr-2">
                {selectedEntryIds.size} of {playlistInfo.available_count} selected
              </span>
              <button
                onClick={handleSelectAvailableOnly}
                className="px-2.5 py-1 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 hover:bg-surface-hover transition-colors"
              >
                Available Only
              </button>
              <button
                onClick={handleSelectAll}
                className="px-2.5 py-1 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 hover:bg-surface-hover transition-colors"
              >
                Select All
              </button>
              <button
                onClick={handleDeselectAll}
                className="px-2.5 py-1 rounded bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 hover:bg-surface-hover transition-colors"
              >
                Clear
              </button>
            </div>
          </div>

          {/* Entry List Table */}
          <div className="max-h-96 overflow-y-auto space-y-2 pr-1 custom-scrollbar">
            {playlistInfo.entries.map((entry) => {
              const isAvailable = entry.availability === "AVAILABLE";
              const isSelected = selectedEntryIds.has(entry.id);

              return (
                <div
                  key={`${entry.id}-${entry.index}`}
                  onClick={() => toggleSelectEntry(entry.id, isAvailable)}
                  className={`p-3 rounded-xl border flex items-center justify-between transition-all ${
                    !isAvailable
                      ? "opacity-50 bg-zinc-100/50 dark:bg-zinc-900/50 border-surface-border cursor-not-allowed"
                      : isSelected
                      ? "bg-brand-500/5 border-brand-500/30 cursor-pointer"
                      : "bg-surface border-surface-border hover:bg-surface-hover cursor-pointer"
                  }`}
                >
                  <div className="flex items-center space-x-3 min-w-0">
                    <button
                      disabled={!isAvailable}
                      className="text-zinc-400 hover:text-brand-500 disabled:cursor-not-allowed"
                    >
                      {isSelected ? (
                        <CheckSquare className="w-4 h-4 text-brand-500" />
                      ) : (
                        <Square className="w-4 h-4 text-zinc-400" />
                      )}
                    </button>

                    <span className="text-xs font-mono text-zinc-400 w-8 flex-shrink-0">
                      #{String(entry.index).padStart(2, "0")}
                    </span>

                    <div className="min-w-0">
                      <h4 className="text-xs font-medium text-zinc-900 dark:text-zinc-100 truncate">
                        {entry.title}
                      </h4>
                      {entry.duration && (
                        <span className="text-[10px] font-mono text-zinc-400">
                          {Math.floor(entry.duration / 60)}:
                          {String(entry.duration % 60).padStart(2, "0")}
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center space-x-2">
                    <span
                      className={`text-[10px] font-mono px-2 py-0.5 rounded ${
                        isAvailable
                          ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                          : "bg-red-500/10 text-red-500"
                      }`}
                    >
                      {entry.availability}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Action Footer */}
          <div className="pt-4 border-t border-surface-border flex flex-col sm:flex-row items-center justify-between gap-4">
            {/* Format Config Controls */}
            <div className="flex items-center space-x-3 text-xs">
              <select
                value={mediaMode}
                onChange={(e) => setMediaMode(e.target.value as MediaMode)}
                className="px-3 py-1.5 rounded-lg bg-surface border border-surface-border font-medium"
              >
                <option value="video">Video (MP4)</option>
                <option value="audio">Audio (MP3)</option>
              </select>

              <select
                value={mediaMode === "video" ? videoQuality : audioQuality}
                onChange={(e) =>
                  mediaMode === "video"
                    ? setVideoQuality(e.target.value as VideoQuality)
                    : setAudioQuality(e.target.value as AudioQuality)
                }
                className="px-3 py-1.5 rounded-lg bg-surface border border-surface-border font-medium"
              >
                {mediaMode === "video" ? (
                  <>
                    <option value="best">Best Quality</option>
                    <option value="2160p">4K (2160p)</option>
                    <option value="1080p">1080p Full HD</option>
                    <option value="720p">720p HD</option>
                  </>
                ) : (
                  <>
                    <option value="best">Best Audio</option>
                    <option value="320k">320 kbps</option>
                    <option value="256k">256 kbps</option>
                  </>
                )}
              </select>
            </div>

            <button
              onClick={handleEnqueuePlaylist}
              disabled={selectedEntryIds.size === 0}
              className="px-6 py-2.5 bg-brand-500 hover:bg-brand-600 text-white rounded-xl text-xs font-semibold flex items-center space-x-2 transition-all disabled:opacity-50 shadow-subtle"
            >
              <Download className="w-4 h-4" />
              <span>Add {selectedEntryIds.size} Selected to Queue →</span>
            </button>
          </div>
        </div>
      )}

      {/* Single Video Download Configuration Form (shown when not inspecting a playlist) */}
      {!playlistInfo && validationResult?.valid && validationResult.url_type !== "PLAYLIST" && (
        <div className="bg-surface-card rounded-2xl border border-surface-border p-6 shadow-subtle space-y-6">
          <div className="border-b border-surface-border pb-3">
            <h2 className="text-base font-semibold text-zinc-900 dark:text-zinc-100">
              Download Settings
            </h2>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {/* Media Mode */}
            <div className="space-y-2">
              <label className="text-xs font-medium text-zinc-700 dark:text-zinc-300">
                Format Mode
              </label>
              <div className="flex space-x-2">
                <button
                  type="button"
                  onClick={() => {
                    setMediaMode("video");
                    setVideoFormat("MP4");
                  }}
                  className={`flex-1 py-2 px-3 rounded-lg border text-xs font-medium flex items-center justify-center space-x-2 transition-all ${
                    mediaMode === "video"
                      ? "bg-brand-500/10 border-brand-500 text-brand-600 dark:text-brand-400"
                      : "bg-surface border-surface-border text-zinc-600"
                  }`}
                >
                  <Video className="w-4 h-4" />
                  <span>Video ({videoFormat})</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setMediaMode("audio");
                    setAudioFormat("MP3");
                  }}
                  className={`flex-1 py-2 px-3 rounded-lg border text-xs font-medium flex items-center justify-center space-x-2 transition-all ${
                    mediaMode === "audio"
                      ? "bg-brand-500/10 border-brand-500 text-brand-600 dark:text-brand-400"
                      : "bg-surface border-surface-border text-zinc-600"
                  }`}
                >
                  <Music className="w-4 h-4" />
                  <span>Audio ({audioFormat})</span>
                </button>
              </div>
            </div>

            {/* Quality Selector */}
            <div className="space-y-2">
              <label className="text-xs font-medium text-zinc-700 dark:text-zinc-300">
                Quality
              </label>
              <select
                value={mediaMode === "video" ? videoQuality : audioQuality}
                onChange={(e) =>
                  mediaMode === "video"
                    ? setVideoQuality(e.target.value as VideoQuality)
                    : setAudioQuality(e.target.value as AudioQuality)
                }
                className="w-full px-3 py-2 bg-surface rounded-lg border border-surface-border text-xs font-medium text-zinc-900 dark:text-zinc-100 outline-none"
              >
                {mediaMode === "video" ? (
                  <>
                    <option value="best">Best Available Quality</option>
                    <option value="2160p">2160p (4K)</option>
                    <option value="1080p">1080p (Full HD)</option>
                    <option value="720p">720p (HD)</option>
                  </>
                ) : (
                  <>
                    <option value="best">Best Audio Stream</option>
                    <option value="320k">320 kbps (High Quality)</option>
                    <option value="256k">256 kbps</option>
                    <option value="192k">192 kbps</option>
                  </>
                )}
              </select>
            </div>
          </div>

          {/* Destination Path Picker */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-zinc-700 dark:text-zinc-300">
              Save Location
            </label>
            <div className="flex items-center space-x-2">
              <input
                type="text"
                readOnly
                value={destinationPath}
                className="flex-1 px-3 py-2 bg-surface rounded-lg border border-surface-border text-xs font-mono text-zinc-600 dark:text-zinc-400 outline-none"
              />
              <button
                type="button"
                onClick={handlePickFolder}
                className="px-3 py-2 rounded-lg border border-surface-border hover:bg-surface-hover text-xs font-medium flex items-center space-x-1.5 transition-colors"
              >
                <Folder className="w-4 h-4 text-zinc-500" />
                <span>Browse</span>
              </button>
            </div>
          </div>

          {/* Progressive Disclosure Toggle */}
          <div className="pt-2">
            <button
              type="button"
              onClick={() => setShowAdvancedDrawer(!showAdvancedDrawer)}
              className="w-full py-2 px-3 rounded-lg border border-slate-700 bg-slate-800/60 hover:bg-slate-800 text-xs font-medium text-slate-300 flex items-center justify-center space-x-1.5 transition-colors"
            >
              <span>{showAdvancedDrawer ? "Advanced Options ▲" : "Advanced Options ▾"}</span>
            </button>
          </div>

          {/* Advanced Options Drawer */}
          <AdvancedOptionsDrawer
            isOpen={showAdvancedDrawer}
            onClose={() => setShowAdvancedDrawer(false)}
            options={downloadOptions}
            onChange={(updated) => setDownloadOptions(updated)}
          />

          {/* Submit Single Download Button */}
          <div className="pt-2">
            <button
              type="button"
              onClick={handleStartSingleDownload}
              className="w-full py-3 bg-brand-500 hover:bg-brand-600 text-white rounded-xl font-semibold text-sm flex items-center justify-center space-x-2 shadow-subtle transition-colors"
            >
              <Download className="w-4 h-4" />
              <span>Download Now</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
