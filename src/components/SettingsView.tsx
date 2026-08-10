import React from "react";
import { Folder, Moon, Sun, Monitor, RefreshCw, FileText } from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { downloadService } from "../services/DownloadService";

export const SettingsView: React.FC = () => {
  const { theme, setTheme, settings, updateSettings } = useAppStore();

  const handleBrowseDefaultFolder = async () => {
    const selected = await downloadService.pickDestinationFolder(settings.defaultDestination);
    if (selected) {
      updateSettings({ defaultDestination: selected });
    }
  };

  return (
    <div className="max-w-3xl mx-auto py-8 px-6 space-y-6">
      <div className="border-b border-surface-border pb-4">
        <h1 className="text-xl font-bold text-zinc-900 dark:text-zinc-100 tracking-tight">
          Settings
        </h1>
        <p className="text-xs text-zinc-500 dark:text-zinc-400">
          Configure Siphonix download behavior and UI preferences.
        </p>
      </div>

      {/* Appearance */}
      <div className="bg-surface-card rounded-xl border border-surface-border p-6 space-y-4">
        <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
          Appearance
        </h2>
        <div className="grid grid-cols-3 gap-3">
          {(["dark", "light", "system"] as const).map((t) => {
            const isActive = theme === t;
            return (
              <button
                key={t}
                onClick={() => setTheme(t)}
                className={`py-3 px-4 rounded-lg border text-xs font-medium capitalize flex items-center justify-center space-x-2 transition-all ${
                  isActive
                    ? "bg-brand-500/10 border-brand-500 text-brand-600 dark:text-brand-400 font-semibold"
                    : "bg-zinc-50 dark:bg-zinc-900 border-surface-border text-zinc-600 dark:text-zinc-400 hover:bg-surface-hover"
                }`}
              >
                {t === "dark" && <Moon className="w-4 h-4" />}
                {t === "light" && <Sun className="w-4 h-4" />}
                {t === "system" && <Monitor className="w-4 h-4" />}
                <span>{t}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Downloads & Concurrency */}
      <div className="bg-surface-card rounded-xl border border-surface-border p-6 space-y-5">
        <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
          Download Settings
        </h2>

        {/* Default Folder */}
        <div className="space-y-2">
          <label className="text-xs font-medium text-zinc-700 dark:text-zinc-300">
            Default Save Location
          </label>
          <div className="flex items-center space-x-2">
            <input
              type="text"
              readOnly
              value={settings.defaultDestination}
              className="w-full py-2 px-3 bg-zinc-50 dark:bg-zinc-900 border border-surface-border rounded-md text-xs font-mono text-zinc-600 dark:text-zinc-400"
            />
            <button
              onClick={handleBrowseDefaultFolder}
              className="px-3 py-2 bg-zinc-200 dark:bg-zinc-800 hover:bg-zinc-300 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-md text-xs font-medium flex items-center space-x-1.5 transition-colors whitespace-nowrap"
            >
              <Folder className="w-3.5 h-3.5" />
              <span>Browse</span>
            </button>
          </div>
        </div>

        {/* Concurrency Limit */}
        <div className="space-y-2">
          <label className="text-xs font-medium text-zinc-700 dark:text-zinc-300 flex items-center justify-between">
            <span>Maximum Concurrent Downloads</span>
            <span className="font-mono text-brand-500 font-semibold">{settings.maxConcurrentDownloads} simultaneous</span>
          </label>
          <div className="grid grid-cols-4 gap-2">
            {[1, 2, 3, 4].map((num) => (
              <button
                key={num}
                onClick={() => updateSettings({ maxConcurrentDownloads: num })}
                className={`py-2 px-3 rounded-md text-xs font-mono font-medium border transition-colors ${
                  settings.maxConcurrentDownloads === num
                    ? "bg-brand-500 text-white border-brand-500 shadow-subtle"
                    : "bg-zinc-50 dark:bg-zinc-900 border-surface-border text-zinc-600 dark:text-zinc-400 hover:bg-surface-hover"
                }`}
              >
                {num} {num === 1 ? "job" : "jobs"}
              </button>
            ))}
          </div>
          <p className="text-[11px] text-zinc-500">
            Conservative concurrency prevents rate limiting and temporary server blocks.
          </p>
        </div>

        {/* Reliability & Auto Retry */}
        <div className="flex items-center justify-between pt-2 border-t border-surface-border">
          <div className="space-y-0.5">
            <label className="text-xs font-medium text-zinc-800 dark:text-zinc-200 flex items-center space-x-1.5">
              <RefreshCw className="w-3.5 h-3.5 text-brand-500" />
              <span>Automatic Retry with Exponential Backoff</span>
            </label>
            <p className="text-[11px] text-zinc-500">
              Automatically retry temporary network glitches without interrupting the queue.
            </p>
          </div>

          <input
            type="checkbox"
            checked={settings.autoRetry}
            onChange={(e) => updateSettings({ autoRetry: e.target.checked })}
            className="w-4 h-4 text-brand-600 rounded border-surface-border focus:ring-brand-500"
          />
        </div>
      </div>

      {/* Diagnostic & Logs */}
      <div className="bg-surface-card rounded-xl border border-surface-border p-6 flex items-center justify-between">
        <div className="space-y-0.5">
          <h3 className="text-xs font-semibold text-zinc-800 dark:text-zinc-200 flex items-center space-x-1.5">
            <FileText className="w-3.5 h-3.5 text-zinc-400" />
            <span>Diagnostics & Activity Logs</span>
          </h3>
          <p className="text-[11px] text-zinc-500">
            View structured application logs for technical details.
          </p>
        </div>

        <button
          onClick={() => alert("Diagnostics log window ready for Phase 1 log viewing.")}
          className="px-3 py-1.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-md text-xs font-medium transition-colors"
        >
          View Logs
        </button>
      </div>
    </div>
  );
};
