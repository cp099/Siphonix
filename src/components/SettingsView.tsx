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

      {/* Runtime Settings Section */}
      <RuntimeSettingsCard />

      {/* Phase 8 Diagnostics & System Health Section */}
      <DiagnosticsAndHealthCard />
    </div>
  );
};

const RuntimeSettingsCard: React.FC = () => {
  const [runtimeStatus, setRuntimeStatus] = React.useState<any>(null);
  const [loading, setLoading] = React.useState(false);
  const [showDiagnostics, setShowDiagnostics] = React.useState(false);

  const fetchStatus = async () => {
    if (downloadService.getRuntimeStatus) {
      setLoading(true);
      try {
        const res = await downloadService.getRuntimeStatus();
        setRuntimeStatus(res);
      } catch (err) {
        console.error(err);
      } finally {
        setLoading(false);
      }
    }
  };

  const handleRefresh = async () => {
    if (downloadService.refreshRuntimeStatus) {
      setLoading(true);
      try {
        const res = await downloadService.refreshRuntimeStatus();
        setRuntimeStatus(res);
      } catch (err) {
        console.error(err);
      } finally {
        setLoading(false);
      }
    }
  };

  React.useEffect(() => {
    fetchStatus();
  }, []);

  if (!runtimeStatus) return null;

  const renderEngineCard = (name: string, info: any) => {
    const isReady = info.status === "READY";
    const sourceLabel =
      info.source === "Managed"
        ? "Siphonix Managed"
        : info.source === "DevOverride"
        ? "Dev Environment Override"
        : "System Installation";

    return (
      <div className="rounded-lg border border-surface-border bg-zinc-50/50 dark:bg-zinc-900/50 p-4 space-y-2">
        <div className="flex items-center justify-between">
          <span className="font-semibold text-xs text-zinc-900 dark:text-zinc-100">{name}</span>
          <span
            className={`px-2 py-0.5 rounded text-[11px] font-semibold ${
              isReady
                ? "bg-emerald-500/10 text-emerald-500"
                : "bg-red-500/10 text-red-500"
            }`}
          >
            {isReady ? "✓ Ready" : "⚠ Not Available"}
          </span>
        </div>

        <div className="space-y-1 text-[11px] text-zinc-600 dark:text-zinc-400">
          <p>Version: <strong className="font-mono text-zinc-800 dark:text-zinc-200">{info.version || "Unknown"}</strong></p>
          <p>Source: <span className="font-medium text-zinc-700 dark:text-zinc-300">{sourceLabel}</span></p>
          {info.path && (
            <p className="truncate font-mono text-[10px] text-zinc-500" title={info.path}>
              Path: {info.path}
            </p>
          )}
        </div>
      </div>
    );
  };

  return (
    <div className="bg-surface-card rounded-xl border border-surface-border p-6 space-y-4">
      <div className="flex items-center justify-between border-b border-surface-border pb-3">
        <div>
          <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
            Runtime Engine Status
          </h2>
          <p className="text-xs text-zinc-500">
            Siphonix-managed and system runtime binary resolution.
          </p>
        </div>

        <button
          onClick={handleRefresh}
          disabled={loading}
          className="flex items-center space-x-1.5 px-3 py-1.5 bg-brand-500/10 hover:bg-brand-500/20 text-brand-600 dark:text-brand-400 rounded-md text-xs font-medium transition-colors"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
          <span>Refresh Status</span>
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {renderEngineCard("yt-dlp", runtimeStatus.yt_dlp)}
        {renderEngineCard("FFmpeg", runtimeStatus.ffmpeg)}
      </div>

      <div className="pt-2">
        <button
          onClick={() => setShowDiagnostics(!showDiagnostics)}
          className="text-xs text-brand-500 font-medium hover:underline flex items-center space-x-1"
        >
          <span>{showDiagnostics ? "Hide Diagnostics ▲" : "View Diagnostics ▾"}</span>
        </button>

        {showDiagnostics && (
          <div className="mt-3 rounded-lg border border-surface-border bg-zinc-900 p-3 text-xs space-y-1.5 font-mono text-zinc-300">
            <p className="text-[11px] font-semibold text-zinc-400 border-b border-zinc-800 pb-1">
              Diagnostic Messages ({runtimeStatus.diagnostics.length}):
            </p>
            {runtimeStatus.diagnostics.map((d: any, idx: number) => (
              <div key={idx} className="flex items-start space-x-2 text-[11px]">
                <span
                  className={`font-bold ${
                    d.level === "error"
                      ? "text-red-400"
                      : d.level === "warning"
                      ? "text-amber-400"
                      : "text-emerald-400"
                  }`}
                >
                  [{d.level.toUpperCase()}]
                </span>
                <span className="text-zinc-200">[{d.code}] {d.message}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

const DiagnosticsAndHealthCard: React.FC = () => {
  const [health, setHealth] = React.useState<any>(null);
  const [reportResult, setReportResult] = React.useState<string | null>(null);
  const [recoveryMsg, setRecoveryMsg] = React.useState<string | null>(null);

  const fetchHealth = async () => {
    if (downloadService.getSystemHealth) {
      try {
        const res = await downloadService.getSystemHealth();
        setHealth(res);
      } catch (err) {
        console.error(err);
      }
    }
  };

  React.useEffect(() => {
    fetchHealth();
  }, []);

  const handleGenerateReport = async () => {
    if (downloadService.generateDiagnosticReport) {
      try {
        const report = await downloadService.generateDiagnosticReport();
        const jsonStr = JSON.stringify(report, null, 2);
        setReportResult(jsonStr);
      } catch (err) {
        alert(`Failed to generate report: ${err}`);
      }
    }
  };

  const handleVerifyLibrary = async () => {
    if (downloadService.verifyLibrary) {
      try {
        const res = await downloadService.verifyLibrary();
        setRecoveryMsg(`[Library] ${res.message}`);
        fetchHealth();
      } catch (err) {
        setRecoveryMsg(`[Library Error] ${err}`);
      }
    }
  };

  const handleVerifyDatabase = async () => {
    if (downloadService.verifyDatabase) {
      try {
        const res = await downloadService.verifyDatabase();
        setRecoveryMsg(`[Database] ${res.message}`);
        fetchHealth();
      } catch (err) {
        setRecoveryMsg(`[Database Error] ${err}`);
      }
    }
  };

  const handleRecoverJobs = async () => {
    if (downloadService.recoverInterruptedJobs) {
      try {
        const res = await downloadService.recoverInterruptedJobs();
        setRecoveryMsg(`[Queue Recovery] ${res.message}`);
        fetchHealth();
      } catch (err) {
        setRecoveryMsg(`[Queue Error] ${err}`);
      }
    }
  };

  return (
    <div className="bg-surface-card rounded-xl border border-surface-border p-6 space-y-5">
      <div className="flex items-center justify-between border-b border-surface-border pb-3">
        <div>
          <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100 flex items-center space-x-2">
            <FileText className="w-4 h-4 text-brand-500" />
            <span>Diagnostics & System Health</span>
          </h2>
          <p className="text-xs text-zinc-500">
            Monitor subsystem integrity and run non-destructive recovery tools.
          </p>
        </div>

        {health && (
          <span
            className={`px-3 py-1 rounded-full text-xs font-bold ${
              health.overall_status === "HEALTHY"
                ? "bg-emerald-500/10 text-emerald-500"
                : health.overall_status === "DEGRADED"
                ? "bg-amber-500/10 text-amber-500"
                : "bg-red-500/10 text-red-500"
            }`}
          >
            {health.overall_status}
          </span>
        )}
      </div>

      {health && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {[
            health.runtime,
            health.database,
            health.queue,
            health.library,
          ].map((sub: any, idx: number) => (
            <div key={idx} className="p-3 rounded-lg border border-surface-border bg-zinc-50/50 dark:bg-zinc-900/50 space-y-1">
              <span className="text-[11px] font-semibold text-zinc-500 uppercase tracking-wider">{sub.name}</span>
              <p
                className={`text-xs font-bold ${
                  sub.status === "HEALTHY"
                    ? "text-emerald-500"
                    : sub.status === "DEGRADED"
                    ? "text-amber-500"
                    : "text-red-500"
                }`}
              >
                {sub.status}
              </p>
            </div>
          ))}
        </div>
      )}

      {recoveryMsg && (
        <div className="p-3 rounded-lg border border-brand-500/30 bg-brand-500/10 text-xs text-brand-400 font-mono">
          {recoveryMsg}
        </div>
      )}

      {/* Action Buttons */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-2 pt-2 border-t border-surface-border">
        <button
          onClick={handleVerifyLibrary}
          className="px-3 py-2 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-md text-xs font-medium transition-colors whitespace-nowrap"
        >
          Verify Library
        </button>

        <button
          onClick={handleVerifyDatabase}
          className="px-3 py-2 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-md text-xs font-medium transition-colors whitespace-nowrap"
        >
          Check Database
        </button>

        <button
          onClick={handleRecoverJobs}
          className="px-3 py-2 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 rounded-md text-xs font-medium transition-colors whitespace-nowrap"
        >
          Recover Queue
        </button>

        <button
          onClick={handleGenerateReport}
          className="px-3 py-2 bg-brand-500 hover:bg-brand-600 text-white rounded-md text-xs font-medium transition-colors whitespace-nowrap"
        >
          Export Report
        </button>
      </div>

      {reportResult && (
        <div className="mt-3 p-3 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-200 font-mono text-[11px] space-y-2">
          <div className="flex items-center justify-between border-b border-zinc-800 pb-1">
            <span className="text-zinc-400 font-bold">Diagnostic Report JSON</span>
            <button
              onClick={() => setReportResult(null)}
              className="text-xs text-zinc-500 hover:text-zinc-300"
            >
              Close
            </button>
          </div>
          <textarea
            readOnly
            value={reportResult}
            rows={8}
            className="w-full bg-transparent text-zinc-300 font-mono text-[10px] outline-none resize-none"
          />
        </div>
      )}
    </div>
  );
};
