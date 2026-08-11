import { create } from "zustand";
import {
  AudioFormat,
  AudioQuality,
  DownloadJob,
  MediaMode,
  UrlValidationResult,
  UserSettings,
  VideoFormat,
  VideoQuality,
} from "../types";
import { downloadService } from "../services/DownloadService";

export type NavTab = "download" | "queue" | "library" | "settings";

interface AppState {
  // Navigation
  activeTab: NavTab;
  setActiveTab: (tab: NavTab) => void;

  // Theme
  theme: "dark" | "light" | "system";
  setTheme: (theme: "dark" | "light" | "system") => void;

  // Active Draft Download Configuration
  url: string;
  setUrl: (url: string) => void;
  inputUrl: string;
  setInputUrl: (url: string) => void;
  urlValidation: UrlValidationResult | null;
  setUrlValidation: (res: UrlValidationResult | null) => void;

  mediaMode: MediaMode;
  setMediaMode: (mode: MediaMode) => void;

  audioFormat: AudioFormat;
  setAudioFormat: (format: AudioFormat) => void;
  audioQuality: AudioQuality;
  setAudioQuality: (quality: AudioQuality) => void;

  videoFormat: VideoFormat;
  setVideoFormat: (format: VideoFormat) => void;
  videoQuality: VideoQuality;
  setVideoQuality: (quality: VideoQuality) => void;

  destinationPath: string;
  setDestinationPath: (path: string) => void;

  // Queue State
  jobs: DownloadJob[];
  setJobs: (jobs: DownloadJob[]) => void;

  // User Settings
  settings: UserSettings;
  updateSettings: (partial: Partial<UserSettings>) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Navigation
  activeTab: "download",
  setActiveTab: (activeTab) => set({ activeTab }),

  // Theme
  theme: "dark",
  setTheme: (theme) => {
    set({ theme });
    if (theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  },

  // Active Draft Configuration
  url: "",
  setUrl: (url) => set({ url, inputUrl: url }),
  inputUrl: "",
  setInputUrl: (inputUrl) => set({ inputUrl, url: inputUrl }),
  urlValidation: null,
  setUrlValidation: (urlValidation) => set({ urlValidation }),

  mediaMode: "video",
  setMediaMode: (mediaMode) => set({ mediaMode }),

  audioFormat: "MP3",
  setAudioFormat: (audioFormat) => set({ audioFormat }),
  audioQuality: "best",
  setAudioQuality: (audioQuality) => set({ audioQuality }),

  videoFormat: "MP4",
  setVideoFormat: (videoFormat) => set({ videoFormat }),
  videoQuality: "1080p",
  setVideoQuality: (videoQuality) => set({ videoQuality }),

  destinationPath: "~/Downloads/Siphonix",
  setDestinationPath: (destinationPath) => set({ destinationPath }),

  // Queue State
  jobs: [],
  setJobs: (jobs) => set({ jobs }),

  // User Settings
  settings: {
    theme: "dark",
    defaultDestination: "~/Downloads/Siphonix",
    maxConcurrentDownloads: 2,
    autoRetry: true,
    maxRetries: 3,
  },
  updateSettings: (partial) =>
    set((state) => ({
      settings: { ...state.settings, ...partial },
    })),
}));

// Subscribe to MockDownloadService queue updates if available
if ("subscribe" in downloadService && typeof (downloadService as any).subscribe === "function") {
  (downloadService as any).subscribe((jobs: DownloadJob[]) => {
    useAppStore.getState().setJobs(jobs);
  });
}
