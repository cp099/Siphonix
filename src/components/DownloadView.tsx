import React, { useState, useEffect } from "react";
import {
  Clipboard,
  CheckCircle2,
  AlertCircle,
  Folder,
  Music,
  Video,
  Sparkles,
  ArrowRight,
  ListVideo,
} from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { downloadService } from "../services/DownloadService";
import { AudioFormat, AudioQuality, VideoFormat, VideoQuality } from "../types";

export const DownloadView: React.FC = () => {
  const {
    inputUrl,
    setInputUrl,
    urlValidation,
    setUrlValidation,
    mediaMode,
    setMediaMode,
    audioFormat,
    setAudioFormat,
    audioQuality,
    setAudioQuality,
    videoFormat,
    setVideoFormat,
    videoQuality,
    setVideoQuality,
    destinationPath,
    setDestinationPath,
    setActiveTab,
  } = useAppStore();

  const [isValidating, setIsValidating] = useState(false);

  // Validate URL whenever inputUrl changes
  useEffect(() => {
    if (!inputUrl.trim()) {
      setUrlValidation(null);
      return;
    }

    const timer = setTimeout(async () => {
      setIsValidating(true);
      try {
        const result = await downloadService.validateUrl(inputUrl);
        setUrlValidation(result);
      } catch (err) {
        setUrlValidation({
          valid: false,
          is_playlist: false,
          video_id: null,
          playlist_id: null,
          message: String(err),
        });
      } finally {
        setIsValidating(false);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [inputUrl, setUrlValidation]);

  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        setInputUrl(text);
      }
    } catch {
      // Fallback
    }
  };

  const handleBrowseFolder = async () => {
    const selected = await downloadService.pickDestinationFolder(destinationPath);
    if (selected) {
      setDestinationPath(selected);
    }
  };

  const handleStartDownload = async () => {
    if (!urlValidation || !urlValidation.valid) return;

    await downloadService.enqueueJob({
      url: inputUrl,
      mediaMode,
      audioFormat: mediaMode === "audio" ? audioFormat : undefined,
      audioQuality: mediaMode === "audio" ? audioQuality : undefined,
      videoFormat: mediaMode === "video" ? videoFormat : undefined,
      videoQuality: mediaMode === "video" ? videoQuality : undefined,
      destinationPath,
      isPlaylist: urlValidation.is_playlist,
    });

    // Reset input & navigate to Queue tab
    setInputUrl("");
    setUrlValidation(null);
    setActiveTab("queue");
  };

  const audioFormats: AudioFormat[] = ["MP3", "M4A", "AAC", "FLAC", "ALAC", "OPUS", "WAV"];
  const videoFormats: VideoFormat[] = ["MP4", "MKV", "WEBM"];

  return (
    <div className="max-w-3xl mx-auto py-8 px-6 space-y-6">
      {/* Hero Header */}
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
          What do you want to download?
        </h1>
        <p className="text-xs text-zinc-500 dark:text-zinc-400">
          Paste a YouTube link below. Siphonix processes audio and video effortlessly.
        </p>
      </div>

      {/* Main Link Input Card */}
      <div className="bg-surface-card rounded-xl border border-surface-border p-4 shadow-subtle space-y-3">
        <div className="relative flex items-center">
          <input
            type="text"
            value={inputUrl}
            onChange={(e) => setInputUrl(e.target.value)}
            placeholder="Paste a YouTube video or playlist link (e.g. https://www.youtube.com/watch?v=...)"
            className="w-full pl-4 pr-24 py-3 bg-zinc-50 dark:bg-zinc-900/90 border border-surface-border rounded-lg text-xs font-mono text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 transition-all"
          />

          <button
            onClick={handlePaste}
            className="absolute right-2 px-3 py-1.5 bg-zinc-200 dark:bg-zinc-800 hover:bg-zinc-300 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-md text-xs font-medium flex items-center space-x-1.5 transition-colors"
          >
            <Clipboard className="w-3.5 h-3.5" />
            <span>Paste</span>
          </button>
        </div>

        {/* Live Validation Indicator */}
        {inputUrl.trim().length > 0 && (
          <div className="text-xs flex items-center space-x-2 pt-1 px-1">
            {isValidating ? (
              <span className="text-zinc-400 animate-pulse">Checking URL...</span>
            ) : urlValidation?.valid ? (
              <div className="text-emerald-500 flex items-center space-x-1.5 font-medium">
                <CheckCircle2 className="w-4 h-4" />
                <span>{urlValidation.message}</span>
                {urlValidation.is_playlist && (
                  <span className="ml-2 px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 text-[10px]">
                    Playlist Detected
                  </span>
                )}
              </div>
            ) : (
              <div className="text-amber-500 flex items-center space-x-1.5">
                <AlertCircle className="w-4 h-4" />
                <span>{urlValidation?.message || "Invalid YouTube link"}</span>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Progressive Disclosure Configuration Card (Revealed when valid URL detected) */}
      {urlValidation?.valid && (
        <div className="bg-surface-card rounded-xl border border-surface-border p-6 shadow-elevated space-y-6 transition-all duration-300">
          <div className="flex items-center justify-between border-b border-surface-border pb-4">
            <div className="flex items-center space-x-2">
              <Sparkles className="w-4 h-4 text-brand-500" />
              <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                Download Preferences
              </h2>
            </div>
            {urlValidation.is_playlist && (
              <div className="flex items-center space-x-1 text-xs text-brand-500 font-medium">
                <ListVideo className="w-4 h-4" />
                <span>Playlist configuration</span>
              </div>
            )}
          </div>

          {/* Media Mode Toggle (Audio vs Video) */}
          <div className="space-y-2">
            <label className="text-xs font-semibold text-zinc-700 dark:text-zinc-300">
              Media Type
            </label>
            <div className="grid grid-cols-2 gap-3">
              <button
                type="button"
                onClick={() => setMediaMode("video")}
                className={`py-3 px-4 rounded-lg border text-xs font-medium flex items-center justify-center space-x-2 transition-all ${
                  mediaMode === "video"
                    ? "bg-brand-500/10 border-brand-500 text-brand-600 dark:text-brand-400 font-semibold shadow-subtle"
                    : "bg-zinc-50 dark:bg-zinc-900 border-surface-border text-zinc-600 dark:text-zinc-400 hover:bg-surface-hover"
                }`}
              >
                <Video className="w-4 h-4" />
                <span>Video</span>
              </button>

              <button
                type="button"
                onClick={() => setMediaMode("audio")}
                className={`py-3 px-4 rounded-lg border text-xs font-medium flex items-center justify-center space-x-2 transition-all ${
                  mediaMode === "audio"
                    ? "bg-brand-500/10 border-brand-500 text-brand-600 dark:text-brand-400 font-semibold shadow-subtle"
                    : "bg-zinc-50 dark:bg-zinc-900 border-surface-border text-zinc-600 dark:text-zinc-400 hover:bg-surface-hover"
                }`}
              >
                <Music className="w-4 h-4" />
                <span>Audio Only</span>
              </button>
            </div>
          </div>

          {/* Format & Quality Selection */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Format Selection */}
            <div className="space-y-2">
              <label className="text-xs font-semibold text-zinc-700 dark:text-zinc-300">
                Format
              </label>
              <div className="flex flex-wrap gap-1.5">
                {mediaMode === "audio"
                  ? audioFormats.map((fmt) => (
                      <button
                        key={fmt}
                        type="button"
                        onClick={() => setAudioFormat(fmt)}
                        className={`px-3 py-1.5 rounded-md text-xs font-mono font-medium border transition-colors ${
                          audioFormat === fmt
                            ? "bg-brand-500 text-white border-brand-500 shadow-subtle"
                            : "bg-zinc-50 dark:bg-zinc-900 border-surface-border text-zinc-600 dark:text-zinc-400 hover:border-zinc-400 dark:hover:border-zinc-600"
                        }`}
                      >
                        {fmt}
                      </button>
                    ))
                  : videoFormats.map((fmt) => (
                      <button
                        key={fmt}
                        type="button"
                        onClick={() => setVideoFormat(fmt)}
                        className={`px-3 py-1.5 rounded-md text-xs font-mono font-medium border transition-colors ${
                          videoFormat === fmt
                            ? "bg-brand-500 text-white border-brand-500 shadow-subtle"
                            : "bg-zinc-50 dark:bg-zinc-900 border-surface-border text-zinc-600 dark:text-zinc-400 hover:border-zinc-400 dark:hover:border-zinc-600"
                        }`}
                      >
                        {fmt}
                      </button>
                    ))}
              </div>
            </div>

            {/* Quality Selection */}
            <div className="space-y-2">
              <label className="text-xs font-semibold text-zinc-700 dark:text-zinc-300">
                Quality
              </label>
              {mediaMode === "audio" ? (
                <select
                  value={audioQuality}
                  onChange={(e) => setAudioQuality(e.target.value as AudioQuality)}
                  className="w-full py-2 px-3 bg-zinc-50 dark:bg-zinc-900 border border-surface-border rounded-md text-xs text-zinc-800 dark:text-zinc-200 focus:border-brand-500"
                >
                  <option value="best">Best available (Recommended)</option>
                  <option value="320k">320 kbps</option>
                  <option value="256k">256 kbps</option>
                  <option value="192k">192 kbps</option>
                  <option value="128k">128 kbps</option>
                </select>
              ) : (
                <select
                  value={videoQuality}
                  onChange={(e) => setVideoQuality(e.target.value as VideoQuality)}
                  className="w-full py-2 px-3 bg-zinc-50 dark:bg-zinc-900 border border-surface-border rounded-md text-xs text-zinc-800 dark:text-zinc-200 focus:border-brand-500"
                >
                  <option value="best">Best available</option>
                  <option value="2160p">2160p (4K Ultra HD)</option>
                  <option value="1440p">1440p (2K QHD)</option>
                  <option value="1080p">1080p (Full HD)</option>
                  <option value="720p">720p (HD)</option>
                  <option value="480p">480p</option>
                  <option value="360p">360p</option>
                </select>
              )}
            </div>
          </div>

          {/* Destination Path Selector */}
          <div className="space-y-2">
            <label className="text-xs font-semibold text-zinc-700 dark:text-zinc-300">
              Save To
            </label>
            <div className="flex items-center space-x-2">
              <input
                type="text"
                readOnly
                value={destinationPath}
                className="w-full py-2 px-3 bg-zinc-50 dark:bg-zinc-900 border border-surface-border rounded-md text-xs font-mono text-zinc-600 dark:text-zinc-400"
              />
              <button
                type="button"
                onClick={handleBrowseFolder}
                className="px-3 py-2 bg-zinc-200 dark:bg-zinc-800 hover:bg-zinc-300 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-md text-xs font-medium flex items-center space-x-1.5 transition-colors whitespace-nowrap"
              >
                <Folder className="w-3.5 h-3.5" />
                <span>Browse</span>
              </button>
            </div>
          </div>

          {/* Download Action Trigger */}
          <div className="pt-2 border-t border-surface-border">
            <button
              onClick={handleStartDownload}
              className="w-full py-3.5 px-6 bg-brand-600 hover:bg-brand-700 text-white rounded-lg font-semibold text-sm shadow-subtle flex items-center justify-center space-x-2 transition-colors"
            >
              <span>
                {urlValidation.is_playlist ? "Download Playlist" : "Download Now"}
              </span>
              <ArrowRight className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
