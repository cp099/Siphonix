import React, { useState, useEffect } from "react";
import {
  Sliders,
  Video,
  Music,
  FolderOutput,
  FileText,
  Subtitles as SubtitleIcon,
  Wifi,
  Bookmark,
  AlertTriangle,
  X,
} from "lucide-react";
import { DownloadOptions, DownloadPreset } from "../types";
import { TauriDownloadService } from "../services/TauriDownloadService";
import { downloadService } from "../services/DownloadService";

interface AdvancedOptionsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  options: DownloadOptions;
  onChange: (updated: DownloadOptions) => void;
}

export const AdvancedOptionsDrawer: React.FC<AdvancedOptionsDrawerProps> = ({
  isOpen,
  onClose,
  options,
  onChange,
}) => {
  const [activeTab, setActiveTab] = useState<"video" | "audio" | "output" | "metadata" | "subtitles" | "network" | "expert">("video");
  const [presets, setPresets] = useState<DownloadPreset[]>([]);
  const [presetNameInput, setPresetNameInput] = useState("");
  const [showSavePresetModal, setShowSavePresetModal] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen && downloadService instanceof TauriDownloadService) {
      downloadService.getPresets().then(setPresets).catch(() => {});
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const updateOptions = (updater: (prev: DownloadOptions) => DownloadOptions) => {
    const next = updater({ ...options });
    // Soft validation preview
    if (next.output.naming_preset === "custom" && next.output.custom_naming_template) {
      if (next.output.custom_naming_template.includes("/") || next.output.custom_naming_template.includes("\\")) {
        setValidationError("Custom template cannot contain path separators.");
      } else {
        setValidationError(null);
      }
    } else {
      setValidationError(null);
    }
    onChange(next);
  };

  const handleLoadPreset = (preset: DownloadPreset) => {
    onChange(preset.options);
    setValidationError(null);
  };

  const handleSavePreset = async () => {
    if (!presetNameInput.trim()) return;
    if (downloadService instanceof TauriDownloadService) {
      try {
        const saved = await downloadService.savePreset(
          undefined,
          presetNameInput.trim(),
          undefined,
          false,
          options
        );
        setPresets((prev) => [...prev, saved]);
        setShowSavePresetModal(false);
        setPresetNameInput("");
      } catch (err: any) {
        setValidationError(err?.toString() || "Failed to save preset");
      }
    }
  };

  return (
    <div className="mt-4 rounded-xl border border-slate-700/60 bg-slate-900/90 p-5 shadow-2xl backdrop-blur-md">
      {/* Drawer Header with Tabs & Presets */}
      <div className="flex flex-col gap-4 border-b border-slate-800 pb-4 md:flex-row md:items-center md:justify-between">
        <div className="flex items-center gap-2 text-indigo-400">
          <Sliders className="h-5 w-5" />
          <h3 className="font-semibold text-white">Advanced Configuration</h3>
          <span className="rounded bg-indigo-500/20 px-2 py-0.5 text-xs text-indigo-300">Phase 6</span>
        </div>

        <div className="flex items-center gap-2">
          {/* Preset Selector */}
          {presets.length > 0 && (
            <select
              onChange={(e) => {
                const found = presets.find((p) => p.id === e.target.value);
                if (found) handleLoadPreset(found);
              }}
              className="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
              defaultValue=""
            >
              <option value="" disabled>Load Preset...</option>
              {presets.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} {p.is_default ? "(Default)" : ""}
                </option>
              ))}
            </select>
          )}

          <button
            onClick={() => setShowSavePresetModal(true)}
            className="flex items-center gap-1.5 rounded-lg border border-slate-700 bg-slate-800/80 px-3 py-1.5 text-xs font-medium text-slate-300 transition hover:bg-slate-700 hover:text-white"
          >
            <Bookmark className="h-3.5 w-3.5" />
            Save Preset
          </button>

          <button
            onClick={onClose}
            className="rounded-lg p-1.5 text-slate-400 hover:bg-slate-800 hover:text-white"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      </div>

      {/* Control Tabs */}
      <div className="mt-4 flex flex-wrap gap-2 border-b border-slate-800 pb-3">
        {[
          { id: "video", label: "Video", icon: Video },
          { id: "audio", label: "Audio", icon: Music },
          { id: "output", label: "Output & Naming", icon: FolderOutput },
          { id: "metadata", label: "Metadata & Assets", icon: FileText },
          { id: "subtitles", label: "Subtitles", icon: SubtitleIcon },
          { id: "network", label: "Network", icon: Wifi },
          { id: "expert", label: "Expert Strategy", icon: Sliders },
        ].map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition ${
                isActive
                  ? "bg-indigo-600 text-white shadow"
                  : "bg-slate-800/50 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
              }`}
            >
              <Icon className="h-3.5 w-3.5" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Validation Error Banner */}
      {validationError && (
        <div className="mt-3 flex items-center gap-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-300">
          <AlertTriangle className="h-4 w-4 shrink-0 text-red-400" />
          <span>{validationError}</span>
        </div>
      )}

      {/* Tab Panels */}
      <div className="mt-4 space-y-4 text-xs">
        {/* VIDEO PANEL */}
        {activeTab === "video" && (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Resolution Cap</label>
              <select
                value={options.video.resolution}
                onChange={(e) => updateOptions((o) => ({ ...o, video: { ...o.video, resolution: e.target.value as any } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="best">Best Available (Source Quality)</option>
                <option value="2160p">2160p (4K)</option>
                <option value="1440p">1440p (2K)</option>
                <option value="1080p">1080p (Full HD)</option>
                <option value="720p">720p (HD)</option>
                <option value="480p">480p (SD)</option>
                <option value="360p">360p</option>
              </select>
            </div>

            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Frame Rate Cap</label>
              <select
                value={options.video.frame_rate}
                onChange={(e) => updateOptions((o) => ({ ...o, video: { ...o.video, frame_rate: e.target.value } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="auto">Auto (Highest Available)</option>
                <option value="60">Up to 60 FPS</option>
                <option value="30">Up to 30 FPS</option>
                <option value="24">Up to 24 FPS</option>
              </select>
            </div>

            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Codec Preference</label>
              <select
                value={options.video.codec_preference}
                onChange={(e) => updateOptions((o) => ({ ...o, video: { ...o.video, codec_preference: e.target.value } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="auto">Auto (Best Efficiency)</option>
                <option value="av1">AV1 (Next-Gen High Efficiency)</option>
                <option value="vp9">VP9 (High Quality YouTube Standard)</option>
                <option value="h264">H.264 / AVC (Maximum Device Compatibility)</option>
              </select>
            </div>

            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Dynamic Range (HDR / SDR)</label>
              <select
                value={options.video.hdr_preference}
                onChange={(e) => updateOptions((o) => ({ ...o, video: { ...o.video, hdr_preference: e.target.value } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="auto">Auto (Source Default)</option>
                <option value="hdr">Prefer HDR</option>
                <option value="sdr">Prefer SDR</option>
              </select>
            </div>

            <div className="md:col-span-2">
              <label className="mb-1.5 block font-medium text-slate-300">Selection Strictness</label>
              <div className="flex gap-4">
                <label className="flex items-center gap-2 text-slate-300 cursor-pointer">
                  <input
                    type="radio"
                    name="selection_mode"
                    value="prefer"
                    checked={options.video.selection_mode === "prefer"}
                    onChange={() => updateOptions((o) => ({ ...o, video: { ...o.video, selection_mode: "prefer" } }))}
                  />
                  <span><strong>Prefer</strong> (Gracefully fall back if requested codec/FPS is unavailable)</span>
                </label>
                <label className="flex items-center gap-2 text-slate-300 cursor-pointer">
                  <input
                    type="radio"
                    name="selection_mode"
                    value="require"
                    checked={options.video.selection_mode === "require"}
                    onChange={() => updateOptions((o) => ({ ...o, video: { ...o.video, selection_mode: "require" } }))}
                  />
                  <span><strong>Require</strong> (Strict filter)</span>
                </label>
              </div>
            </div>
          </div>
        )}

        {/* AUDIO PANEL */}
        {activeTab === "audio" && (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Audio Container / Format</label>
              <select
                value={options.audio.format}
                onChange={(e) => updateOptions((o) => ({ ...o, audio: { ...o.audio, format: e.target.value as any } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="MP3 font-bold">MP3 (Universal Compatibility)</option>
                <option value="M4A">M4A (AAC Stream - Apple Compatible)</option>
                <option value="AAC">AAC (High Compression)</option>
                <option value="OPUS">OPUS (Modern High-Efficiency Audio)</option>
                <option value="FLAC">FLAC (Lossless Compressed)</option>
                <option value="ALAC">ALAC (Apple Lossless)</option>
                <option value="WAV">WAV (Uncompressed Audio)</option>
              </select>
            </div>

            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Target Bitrate / Quality</label>
              <select
                value={options.audio.quality}
                onChange={(e) => updateOptions((o) => ({ ...o, audio: { ...o.audio, quality: e.target.value as any } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="best">Best VBR Quality</option>
                <option value="320k">320 kbps (High Quality)</option>
                <option value="256k">256 kbps</option>
                <option value="192k">192 kbps</option>
                <option value="128k">128 kbps</option>
              </select>
            </div>
          </div>
        )}

        {/* OUTPUT & NAMING PANEL */}
        {activeTab === "output" && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div>
                <label className="mb-1.5 block font-medium text-slate-300">Container Format</label>
                <select
                  value={options.output.container}
                  onChange={(e) => updateOptions((o) => ({ ...o, output: { ...o.output, container: e.target.value } }))}
                  className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
                >
                  <option value="MP4">MP4 (Best Compatibility)</option>
                  <option value="MKV">MKV (Flexible Container for Subtitles & Audio)</option>
                  <option value="WEBM">WebM (Web Optimized)</option>
                </select>
              </div>

              <div>
                <label className="mb-1.5 block font-medium text-slate-300">Folder Organization</label>
                <select
                  value={options.output.folder_organization}
                  onChange={(e) => updateOptions((o) => ({ ...o, output: { ...o.output, folder_organization: e.target.value } }))}
                  className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
                >
                  <option value="flat">Flat (Single Output Directory)</option>
                  <option value="by_playlist">By Playlist (Downloads/Playlist Name/Video.mp4)</option>
                  <option value="by_channel">By Channel (Downloads/Channel Name/Video.mp4)</option>
                </select>
              </div>
            </div>

            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Filename Naming Template</label>
              <select
                value={options.output.naming_preset}
                onChange={(e) => updateOptions((o) => ({ ...o, output: { ...o.output, naming_preset: e.target.value } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="simple">Simple: Title.ext</option>
                <option value="title_id">Title + ID: Title [ID].ext</option>
                <option value="artist_title">Artist + Title: Artist - Title.ext</option>
                <option value="playlist_index">Playlist Index: 01 - Title.ext</option>
                <option value="custom">Custom Template...</option>
              </select>
            </div>

            {options.output.naming_preset === "custom" && (
              <div>
                <label className="mb-1.5 block font-medium text-slate-300">Custom Template Pattern</label>
                <input
                  type="text"
                  value={options.output.custom_naming_template || ""}
                  onChange={(e) => updateOptions((o) => ({ ...o, output: { ...o.output, custom_naming_template: e.target.value } }))}
                  placeholder="%(playlist_index)02d - %(title)s [%(id)s].%(ext)s"
                  className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200 font-mono"
                />
                <p className="mt-1 text-[11px] text-slate-400">
                  Allowed tokens: <code>%(title)s</code>, <code>%(id)s</code>, <code>%(artist)s</code>, <code>%(playlist_index)02d</code>, <code>%(ext)s</code>
                </p>
              </div>
            )}

            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Overwrite Policy</label>
              <select
                value={options.output.overwrite_policy}
                onChange={(e) => updateOptions((o) => ({ ...o, output: { ...o.output, overwrite_policy: e.target.value } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="ask">Ask Confirmation Before Overwriting (Recommended)</option>
                <option value="never">Never Overwrite (Skip Existing Files)</option>
                <option value="replace">Always Replace Existing Files</option>
              </select>
            </div>
          </div>
        )}

        {/* METADATA PANEL */}
        {activeTab === "metadata" && (
          <div className="space-y-3">
            <label className="flex items-center gap-2 text-slate-300 cursor-pointer">
              <input
                type="checkbox"
                checked={options.metadata.embed_metadata}
                onChange={(e) => updateOptions((o) => ({ ...o, metadata: { ...o.metadata, embed_metadata: e.target.checked } }))}
                className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500"
              />
              <span>Embed Title, Artist, Album & Video Tags</span>
            </label>

            <label className="flex items-center gap-2 text-slate-300 cursor-pointer">
              <input
                type="checkbox"
                checked={options.metadata.embed_thumbnail}
                onChange={(e) => updateOptions((o) => ({ ...o, metadata: { ...o.metadata, embed_thumbnail: e.target.checked } }))}
                className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500"
              />
              <span>Embed Cover Art Thumbnail into Audio / Video File</span>
            </label>

            <label className="flex items-center gap-2 text-slate-300 cursor-pointer">
              <input
                type="checkbox"
                checked={options.metadata.write_metadata_json}
                onChange={(e) => updateOptions((o) => ({ ...o, metadata: { ...o.metadata, write_metadata_json: e.target.checked } }))}
                className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500"
              />
              <span>Save Metadata JSON Sidecar File (.info.json)</span>
            </label>
          </div>
        )}

        {/* SUBTITLES PANEL */}
        {activeTab === "subtitles" && (
          <div className="space-y-4">
            <label className="flex items-center gap-2 text-slate-300 cursor-pointer">
              <input
                type="checkbox"
                checked={options.subtitles.enabled}
                onChange={(e) => updateOptions((o) => ({ ...o, subtitles: { ...o.subtitles, enabled: e.target.checked } }))}
                className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500"
              />
              <span className="font-semibold text-white">Enable Subtitle Download</span>
            </label>

            {options.subtitles.enabled && (
              <div className="space-y-3 pl-6 border-l-2 border-indigo-500/30">
                <div>
                  <label className="mb-1.5 block font-medium text-slate-300">Subtitle Format</label>
                  <select
                    value={options.subtitles.format}
                    onChange={(e) => updateOptions((o) => ({ ...o, subtitles: { ...o.subtitles, format: e.target.value } }))}
                    className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
                  >
                    <option value="srt">SubRip (.srt)</option>
                    <option value="vtt">WebVTT (.vtt)</option>
                    <option value="ass">Advanced SubStation Alpha (.ass)</option>
                  </select>
                </div>

                <label className="flex items-center gap-2 text-slate-300 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={options.subtitles.embed_in_video}
                    onChange={(e) => updateOptions((o) => ({ ...o, subtitles: { ...o.subtitles, embed_in_video: e.target.checked } }))}
                    className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500"
                  />
                  <span>Embed Subtitles Directly into Video Track</span>
                </label>
              </div>
            )}
          </div>
        )}

        {/* NETWORK PANEL */}
        {activeTab === "network" && (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Fragment Concurrency</label>
              <select
                value={options.network.concurrent_fragments}
                onChange={(e) => updateOptions((o) => ({ ...o, network: { ...o.network, concurrent_fragments: e.target.value } }))}
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
              >
                <option value="auto">Auto (Engine Default)</option>
                <option value="1">1 (Single Fragment)</option>
                <option value="2">2 Concurrent Fragments</option>
                <option value="4">4 Concurrent Fragments</option>
                <option value="8">8 Concurrent Fragments</option>
              </select>
            </div>

            <div>
              <label className="mb-1.5 block font-medium text-slate-300">Download Speed Limit</label>
              <input
                type="text"
                value={options.network.max_download_rate || ""}
                onChange={(e) => updateOptions((o) => ({ ...o, network: { ...o.network, max_download_rate: e.target.value || undefined } }))}
                placeholder="Unlimited (e.g. 10M or 500K)"
                className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200 font-mono"
              />
            </div>
          </div>
        )}

        {/* EXPERT STRATEGY PANEL */}
        {activeTab === "expert" && (
          <div>
            <label className="mb-1.5 block font-medium text-slate-300">Quality & Format Sorting Priority</label>
            <select
              value={options.expert.format_sort_strategy}
              onChange={(e) => updateOptions((o) => ({ ...o, expert: { ...o.expert, format_sort_strategy: e.target.value } }))}
              className="w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-200"
            >
              <option value="resolution_first">Resolution First (Height $\rightarrow$ Codec $\rightarrow$ FPS)</option>
              <option value="codec_first">Codec First (AV1/VP9 $\rightarrow$ Resolution $\rightarrow$ FPS)</option>
              <option value="fps_first">Frame Rate First (60 FPS $\rightarrow$ Resolution $\rightarrow$ Codec)</option>
              <option value="size_first">File Size First (Smallest File Size $\rightarrow$ Resolution)</option>
              <option value="audio_first">Audio Quality First (Highest Bitrate $\rightarrow$ Resolution)</option>
            </select>
          </div>
        )}
      </div>

      {/* SAVE PRESET MODAL */}
      {showSavePresetModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm">
          <div className="w-full max-w-sm rounded-xl border border-slate-700 bg-slate-900 p-5 shadow-2xl">
            <h4 className="font-semibold text-white text-sm">Save Custom Preset</h4>
            <input
              type="text"
              value={presetNameInput}
              onChange={(e) => setPresetNameInput(e.target.value)}
              placeholder="Preset Name (e.g., 4K Archival)"
              className="mt-3 w-full rounded-lg border border-slate-700 bg-slate-800 p-2 text-xs text-white"
            />
            <div className="mt-4 flex justify-end gap-2">
              <button
                onClick={() => setShowSavePresetModal(false)}
                className="rounded-lg border border-slate-700 px-3 py-1.5 text-xs text-slate-300 hover:bg-slate-800"
              >
                Cancel
              </button>
              <button
                onClick={handleSavePreset}
                className="rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-indigo-500"
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
