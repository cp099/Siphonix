import React, { useEffect, useState } from "react";
import { Moon, Sun, ShieldCheck } from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { downloadService } from "../services/DownloadService";
import { AppInfo } from "../types";

export const Header: React.FC = () => {
  const { theme, setTheme } = useAppStore();
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    downloadService.getAppInfo().then(setAppInfo).catch(console.error);
  }, []);

  const toggleTheme = () => {
    if (theme === "dark") setTheme("light");
    else setTheme("dark");
  };

  return (
    <header className="h-16 border-b border-surface-border bg-surface-card flex items-center justify-between px-6 select-none">
      {/* Brand Identity */}
      <div className="flex items-center space-x-3">
        {/* Geometric Siphon Icon */}
        <div className="w-9 h-9 rounded-lg bg-gradient-to-tr from-brand-600 to-indigo-500 flex items-center justify-center shadow-subtle">
          <svg
            className="w-5 h-5 text-white stroke-[2.2]"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M4 12v-2a4 4 0 0 1 4-4h8a4 4 0 0 1 4 4v2" />
            <path d="m18 9 3 3-3 3" />
            <path d="M20 12v2a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4v-2" />
            <path d="m6 15-3-3 3-3" />
          </svg>
        </div>

        <div>
          <div className="flex items-baseline space-x-2">
            <span className="font-bold text-lg tracking-tight text-zinc-900 dark:text-zinc-100">
              Siphonix
            </span>
            <span className="text-xs font-mono text-zinc-500 dark:text-zinc-400">
              v{appInfo?.version || "0.1.0"}
            </span>
          </div>
          <p className="text-xs font-medium text-zinc-500 dark:text-zinc-400">
            Make it yours.
          </p>
        </div>
      </div>

      {/* Action Toolbar */}
      <div className="flex items-center space-x-3">
        {/* Status Badge */}
        <div className="hidden sm:flex items-center space-x-1.5 px-2.5 py-1 rounded-full bg-zinc-100 dark:bg-zinc-800/60 border border-surface-border text-xs text-zinc-600 dark:text-zinc-300">
          <ShieldCheck className="w-3.5 h-3.5 text-emerald-500" />
          <span className="capitalize">{appInfo?.platform || "Windows x64"}</span>
        </div>

        {/* Theme Toggle Button */}
        <button
          onClick={toggleTheme}
          aria-label="Toggle theme"
          className="p-2 rounded-md hover:bg-surface-hover text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-100 transition-colors"
        >
          {theme === "dark" ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
        </button>
      </div>
    </header>
  );
};
