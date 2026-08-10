import React, { useEffect, useState } from "react";
import { Folder, CheckCircle2, Inbox } from "lucide-react";
import { DownloadJob } from "../types";
import { downloadService } from "../services/DownloadService";
import { TauriDownloadService } from "../services/TauriDownloadService";

export const LibraryView: React.FC = () => {
  const [libraryJobs, setLibraryJobs] = useState<DownloadJob[]>([]);

  useEffect(() => {
    const fetchLibrary = async () => {
      if (downloadService instanceof TauriDownloadService) {
        const jobs = await downloadService.getLibraryJobs();
        setLibraryJobs(jobs);
      }
    };
    fetchLibrary();
  }, []);

  return (
    <div className="max-w-4xl mx-auto py-8 px-6 space-y-6">
      <div className="flex items-center justify-between border-b border-surface-border pb-4">
        <div>
          <h1 className="text-xl font-bold text-zinc-900 dark:text-zinc-100 tracking-tight">
            Library
          </h1>
          <p className="text-xs text-zinc-500 dark:text-zinc-400">
            Completed downloads stored in persistent SQLite storage.
          </p>
        </div>
      </div>

      {libraryJobs.length === 0 ? (
        <div className="bg-surface-card rounded-xl border border-surface-border p-12 text-center space-y-3">
          <div className="w-12 h-12 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center mx-auto text-zinc-400">
            <Inbox className="w-6 h-6" />
          </div>
          <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
            No completed downloads yet
          </h3>
          <p className="text-xs text-zinc-500 max-w-sm mx-auto">
            Once your downloads complete, they will appear here in your library.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {libraryJobs.map((item) => (
            <div
              key={item.id}
              className="bg-surface-card rounded-xl border border-surface-border p-4 flex items-center justify-between shadow-subtle hover:border-zinc-300 dark:hover:border-zinc-700 transition-colors"
            >
              <div className="flex items-center space-x-3">
                <div className="w-10 h-10 rounded-lg bg-emerald-500/10 text-emerald-500 flex items-center justify-center flex-shrink-0">
                  <CheckCircle2 className="w-5 h-5" />
                </div>
                <div>
                  <h4 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                    {item.title}
                  </h4>
                  <div className="flex items-center space-x-2 text-xs font-mono text-zinc-400">
                    <span>{item.format}</span>
                    <span>•</span>
                    <span>{item.quality}</span>
                    <span>•</span>
                    <span className="truncate max-w-xs">{item.destinationPath}</span>
                  </div>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                <button
                  onClick={() => alert(`File saved at: ${item.destinationPath}`)}
                  className="px-3 py-1.5 rounded-lg bg-zinc-100 dark:bg-zinc-800 hover:bg-surface-hover text-zinc-700 dark:text-zinc-300 text-xs font-medium flex items-center space-x-1.5 transition-colors"
                >
                  <Folder className="w-3.5 h-3.5" />
                  <span>Show in Folder</span>
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
