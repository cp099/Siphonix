import React from "react";
import { Header } from "./components/Header";
import { Navigation } from "./components/Navigation";
import { DownloadView } from "./components/DownloadView";
import { QueueView } from "./components/QueueView";
import { LibraryView } from "./components/LibraryView";
import { SettingsView } from "./components/SettingsView";
import { useAppStore } from "./store/useAppStore";

export const App: React.FC = () => {
  const { activeTab } = useAppStore();

  return (
    <div className="min-h-screen bg-surface-base text-zinc-900 dark:text-zinc-100 flex flex-col font-sans transition-colors duration-200">
      {/* Top Application Header */}
      <Header />

      {/* Main Tab Navigation */}
      <Navigation />

      {/* View Router */}
      <main className="flex-1 overflow-y-auto">
        {activeTab === "download" && <DownloadView />}
        {activeTab === "queue" && <QueueView />}
        {activeTab === "library" && <LibraryView />}
        {activeTab === "settings" && <SettingsView />}
      </main>
    </div>
  );
};

export default App;
