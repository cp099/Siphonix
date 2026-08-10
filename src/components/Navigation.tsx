import React from "react";
import { Download, ListVideo, Library as LibraryIcon, Settings as SettingsIcon } from "lucide-react";
import { useAppStore, NavTab } from "../store/useAppStore";

export const Navigation: React.FC = () => {
  const { activeTab, setActiveTab, jobs } = useAppStore();

  const activeCount = jobs.filter(
    (j) => j.state === "QUEUED" || j.state === "DOWNLOADING" || j.state === "PREPARING" || j.state === "PROCESSING"
  ).length;

  const tabs: { id: NavTab; label: string; icon: React.FC<{ className?: string }>; badge?: number }[] = [
    { id: "download", label: "Download", icon: Download },
    { id: "queue", label: "Queue", icon: ListVideo, badge: activeCount },
    { id: "library", label: "Library", icon: LibraryIcon },
    { id: "settings", label: "Settings", icon: SettingsIcon },
  ];

  return (
    <nav className="w-full bg-surface-card border-b border-surface-border px-6 flex items-center justify-between select-none">
      <div className="flex space-x-1">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;

          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`relative py-3.5 px-4 text-xs font-medium flex items-center space-x-2 transition-all border-b-2 ${
                isActive
                  ? "border-brand-500 text-brand-600 dark:text-brand-500 font-semibold"
                  : "border-transparent text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200"
              }`}
            >
              <Icon className={`w-4 h-4 ${isActive ? "text-brand-500" : "text-zinc-400"}`} />
              <span>{tab.label}</span>
              {typeof tab.badge === "number" && tab.badge > 0 && (
                <span className="ml-1.5 px-1.5 py-0.5 text-[10px] font-bold rounded-full bg-brand-500 text-white">
                  {tab.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>
    </nav>
  );
};
