import React, { useEffect, useState } from "react";
import {
  Folder,
  Play,
  Search,
  RefreshCw,
  MoreVertical,
  Trash2,
  BookmarkX,
  Copy,
  Check,
  Video,
  Music,
  AlertTriangle,
  Inbox,
  ListMusic,
} from "lucide-react";
import { LibraryItem } from "../types";
import { downloadService } from "../services/DownloadService";
import { TauriDownloadService } from "../services/TauriDownloadService";

export const LibraryView: React.FC = () => {
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [search, setSearch] = useState("");
  const [filterTab, setFilterTab] = useState<"ALL" | "video" | "audio" | "MISSING">("ALL");
  const [sortBy, setSortBy] = useState<"newest" | "oldest" | "title_asc" | "size_desc">("newest");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [openMenuId, setOpenMenuId] = useState<string | null>(null);
  const [deleteModalItem, setDeleteModalItem] = useState<LibraryItem | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const fetchLibrary = async () => {
    if (downloadService instanceof TauriDownloadService) {
      const modeArg = filterTab === "video" || filterTab === "audio" ? filterTab : "ALL";
      const statusArg = filterTab === "MISSING" ? "MISSING" : "ALL";
      const result = await downloadService.getLibraryItems(search, modeArg, statusArg, sortBy);
      setItems(result);
    }
  };

  useEffect(() => {
    fetchLibrary();
  }, [search, filterTab, sortBy]);

  const handleRefreshStatus = async () => {
    if (downloadService instanceof TauriDownloadService) {
      setIsRefreshing(true);
      try {
        await downloadService.verifyLibraryStatus();
        await fetchLibrary();
      } finally {
        setIsRefreshing(false);
      }
    }
  };

  const handleOpen = async (item: LibraryItem) => {
    if (downloadService instanceof TauriDownloadService) {
      try {
        await downloadService.openLibraryItem(item.id);
      } catch (err: any) {
        alert(err.toString());
        await fetchLibrary();
      }
    }
  };

  const handleReveal = async (item: LibraryItem) => {
    if (downloadService instanceof TauriDownloadService) {
      try {
        await downloadService.revealLibraryItem(item.id);
      } catch (err: any) {
        alert(err.toString());
        await fetchLibrary();
      }
    }
  };

  const handleCopyUrl = (item: LibraryItem) => {
    navigator.clipboard.writeText(item.sourceUrl);
    setCopiedId(item.id);
    setTimeout(() => setCopiedId(null), 2000);
    setOpenMenuId(null);
  };

  const handleRemoveFromLibrary = async (item: LibraryItem) => {
    if (downloadService instanceof TauriDownloadService) {
      await downloadService.removeLibraryItem(item.id);
      setOpenMenuId(null);
      await fetchLibrary();
    }
  };

  const handleConfirmDeleteFile = async () => {
    if (!deleteModalItem || !(downloadService instanceof TauriDownloadService)) return;
    setIsDeleting(true);
    try {
      await downloadService.deleteLibraryFile(deleteModalItem.id);
      setDeleteModalItem(null);
      await fetchLibrary();
    } catch (err: any) {
      alert(err.toString());
    } finally {
      setIsDeleting(false);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes <= 0) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  };

  return (
    <div className="max-w-5xl mx-auto py-8 px-6 space-y-6">
      {/* Header Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between border-b border-surface-border pb-4 gap-4">
        <div>
          <h1 className="text-xl font-bold text-zinc-900 dark:text-zinc-100 tracking-tight">
            Library
          </h1>
          <p className="text-xs text-zinc-500 dark:text-zinc-400">
            Local media files managed by Siphonix.
          </p>
        </div>

        <div className="flex items-center space-x-3">
          <button
            onClick={handleRefreshStatus}
            disabled={isRefreshing}
            className="px-3 py-1.5 rounded-lg bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 text-xs font-medium flex items-center space-x-1.5 transition-colors disabled:opacity-50"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
            <span>{isRefreshing ? "Verifying..." : "Refresh Status"}</span>
          </button>
        </div>
      </div>

      {/* Search, Filter & Sort Controls */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        {/* Search Box */}
        <div className="relative flex-1 max-w-md">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-400" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search downloads by title, filename, or playlist..."
            className="w-full pl-9 pr-4 py-2 bg-surface-card border border-surface-border rounded-xl text-xs text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-accent-primary/50 transition-all"
          />
        </div>

        {/* Filter Tabs & Sort Dropdown */}
        <div className="flex items-center space-x-3 overflow-x-auto pb-1 sm:pb-0">
          <div className="flex items-center p-1 bg-surface-card border border-surface-border rounded-xl">
            {(["ALL", "video", "audio", "MISSING"] as const).map((tab) => (
              <button
                key={tab}
                onClick={() => setFilterTab(tab)}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium capitalize transition-all ${
                  filterTab === tab
                    ? "bg-accent-primary text-white shadow-subtle"
                    : "text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-200"
                }`}
              >
                {tab === "ALL" ? "All" : tab === "MISSING" ? "Missing" : tab}
              </button>
            ))}
          </div>

          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="px-3 py-2 bg-surface-card border border-surface-border rounded-xl text-xs text-zinc-700 dark:text-zinc-300 focus:outline-none focus:ring-2 focus:ring-accent-primary/50"
          >
            <option value="newest">Newest First</option>
            <option value="oldest">Oldest First</option>
            <option value="title_asc">Name (A–Z)</option>
            <option value="size_desc">Size (Largest)</option>
          </select>
        </div>
      </div>

      {/* Library Grid / List */}
      {items.length === 0 ? (
        <div className="bg-surface-card rounded-xl border border-surface-border p-12 text-center space-y-3">
          <div className="w-12 h-12 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center mx-auto text-zinc-400">
            <Inbox className="w-6 h-6" />
          </div>
          <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
            No library items found
          </h3>
          <p className="text-xs text-zinc-500 max-w-sm mx-auto">
            {search || filterTab !== "ALL"
              ? "No files match your current search or filter criteria."
              : "Completed media downloads will automatically appear in your local library."}
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {items.map((item) => (
            <div
              key={item.id}
              className={`bg-surface-card rounded-xl border p-4 flex flex-col md:flex-row md:items-center justify-between shadow-subtle transition-all relative ${
                item.fileStatus === "MISSING"
                  ? "border-amber-500/30 dark:border-amber-500/20 bg-amber-500/5"
                  : "border-surface-border hover:border-zinc-300 dark:hover:border-zinc-700"
              }`}
            >
              {/* Main Information */}
              <div className="flex items-start space-x-3.5 mb-3 md:mb-0 min-w-0 flex-1 pr-4">
                {/* Media Thumbnail / Icon */}
                <div className="relative w-14 h-14 rounded-lg bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center flex-shrink-0 overflow-hidden border border-surface-border">
                  {item.thumbnailUrl ? (
                    <img
                      src={item.thumbnailUrl}
                      alt={item.title}
                      className="w-full h-full object-cover"
                      onError={(e) => {
                        (e.target as HTMLElement).style.display = "none";
                      }}
                    />
                  ) : item.mediaMode === "video" ? (
                    <Video className="w-6 h-6 text-zinc-400" />
                  ) : (
                    <Music className="w-6 h-6 text-zinc-400" />
                  )}
                </div>

                <div className="space-y-1 min-w-0 flex-1">
                  <div className="flex items-center space-x-2 flex-wrap">
                    <h4 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100 truncate max-w-lg">
                      {item.title}
                    </h4>

                    {/* Status Badge */}
                    {item.fileStatus === "MISSING" ? (
                      <span className="px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-600 dark:text-amber-400 text-[10px] font-semibold uppercase tracking-wider flex items-center space-x-1">
                        <AlertTriangle className="w-3 h-3" />
                        <span>Missing</span>
                      </span>
                    ) : (
                      <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 text-[10px] font-semibold uppercase tracking-wider">
                        Available
                      </span>
                    )}
                  </div>

                  {/* Metadata Pill */}
                  <div className="flex items-center space-x-2 text-xs font-mono text-zinc-500 dark:text-zinc-400 flex-wrap gap-y-1">
                    <span className="uppercase font-semibold">{item.format}</span>
                    <span>•</span>
                    <span>{item.quality}</span>
                    <span>•</span>
                    <span>{formatFileSize(item.fileSizeBytes)}</span>
                    <span>•</span>
                    <span className="truncate max-w-xs text-zinc-400" title={item.filePath}>
                      {item.fileName}
                    </span>
                  </div>

                  {/* Playlist Provenance Badge */}
                  {item.sourcePlaylistTitle && (
                    <div className="flex items-center space-x-1 text-[11px] text-accent-primary font-medium">
                      <ListMusic className="w-3 h-3" />
                      <span>
                        From playlist: {item.sourcePlaylistTitle}
                        {item.playlistEntryIndex ? ` • #${item.playlistEntryIndex}` : ""}
                      </span>
                    </div>
                  )}
                </div>
              </div>

              {/* Action Buttons */}
              <div className="flex items-center space-x-2 justify-end">
                {item.fileStatus === "AVAILABLE" && (
                  <>
                    <button
                      onClick={() => handleOpen(item)}
                      className="px-3 py-1.5 rounded-lg bg-accent-primary hover:bg-accent-primary-hover text-white text-xs font-medium flex items-center space-x-1.5 shadow-subtle transition-all"
                    >
                      <Play className="w-3.5 h-3.5 fill-current" />
                      <span>Open</span>
                    </button>

                    <button
                      onClick={() => handleReveal(item)}
                      className="px-3 py-1.5 rounded-lg bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 text-xs font-medium flex items-center space-x-1.5 transition-colors"
                    >
                      <Folder className="w-3.5 h-3.5" />
                      <span>Show in Folder</span>
                    </button>
                  </>
                )}

                {/* More Options Dropdown */}
                <div className="relative">
                  <button
                    onClick={() => setOpenMenuId(openMenuId === item.id ? null : item.id)}
                    className="p-1.5 rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-500 transition-colors"
                  >
                    <MoreVertical className="w-4 h-4" />
                  </button>

                  {openMenuId === item.id && (
                    <div className="absolute right-0 mt-1 w-48 bg-surface-card border border-surface-border rounded-xl shadow-lg py-1 z-20 text-xs">
                      <button
                        onClick={() => handleCopyUrl(item)}
                        className="w-full px-3 py-2 text-left hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-700 dark:text-zinc-300 flex items-center space-x-2"
                      >
                        {copiedId === item.id ? (
                          <Check className="w-3.5 h-3.5 text-emerald-500" />
                        ) : (
                          <Copy className="w-3.5 h-3.5 text-zinc-400" />
                        )}
                        <span>{copiedId === item.id ? "Copied Link!" : "Copy Source URL"}</span>
                      </button>

                      <button
                        onClick={() => handleRemoveFromLibrary(item)}
                        className="w-full px-3 py-2 text-left hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-700 dark:text-zinc-300 flex items-center space-x-2"
                      >
                        <BookmarkX className="w-3.5 h-3.5 text-zinc-400" />
                        <span>Remove from Library</span>
                      </button>

                      <div className="my-1 border-t border-surface-border" />

                      <button
                        onClick={() => {
                          setDeleteModalItem(item);
                          setOpenMenuId(null);
                        }}
                        className="w-full px-3 py-2 text-left hover:bg-red-500/10 text-red-600 dark:text-red-400 flex items-center space-x-2 font-medium"
                      >
                        <Trash2 className="w-3.5 h-3.5 text-red-500" />
                        <span>Delete File...</span>
                      </button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {deleteModalItem && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 z-50 animate-in fade-in duration-200">
          <div className="bg-surface-card border border-surface-border rounded-2xl p-6 max-w-md w-full shadow-2xl space-y-4">
            <div className="flex items-center space-x-3 text-red-500">
              <div className="w-10 h-10 rounded-full bg-red-500/10 flex items-center justify-center flex-shrink-0">
                <Trash2 className="w-5 h-5" />
              </div>
              <div>
                <h3 className="text-base font-bold text-zinc-900 dark:text-zinc-100">
                  Delete Media File?
                </h3>
                <p className="text-xs text-zinc-500">
                  This action permanently deletes the file from disk.
                </p>
              </div>
            </div>

            <div className="bg-zinc-100 dark:bg-zinc-800/60 rounded-xl p-3 text-xs space-y-1 font-mono text-zinc-600 dark:text-zinc-300 break-all">
              <p className="font-semibold text-zinc-900 dark:text-zinc-100">{deleteModalItem.title}</p>
              <p className="text-[11px] text-zinc-400">{deleteModalItem.filePath}</p>
            </div>

            <p className="text-xs text-zinc-500 leading-relaxed">
              Are you sure you want to delete this file? This cannot be undone. The download history job entry will remain intact.
            </p>

            <div className="flex items-center justify-end space-x-3 pt-2">
              <button
                onClick={() => setDeleteModalItem(null)}
                disabled={isDeleting}
                className="px-4 py-2 rounded-xl bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-700 dark:text-zinc-300 text-xs font-medium transition-colors disabled:opacity-50"
              >
                Cancel
              </button>

              <button
                onClick={handleConfirmDeleteFile}
                disabled={isDeleting}
                className="px-4 py-2 rounded-xl bg-red-600 hover:bg-red-700 text-white text-xs font-medium shadow-subtle transition-all disabled:opacity-50 flex items-center space-x-1.5"
              >
                {isDeleting && <RefreshCw className="w-3.5 h-3.5 animate-spin" />}
                <span>{isDeleting ? "Deleting..." : "Delete File"}</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
