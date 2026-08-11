import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { AppInfo, UrlValidationResult } from "../types";

/**
 * Safe check to determine whether the app is executing inside a Tauri window.
 */
export function isTauriEnv(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Invoke Rust IPC command validate_url with browser fallback.
 */
export async function invokeValidateUrl(url: string): Promise<UrlValidationResult> {
  if (isTauriEnv()) {
    try {
      return await invoke<UrlValidationResult>("validate_url", { url });
    } catch (err) {
      return {
        valid: false,
        url_type: "INVALID",
        is_playlist: false,
        video_id: null,
        playlist_id: null,
        message: err instanceof Error ? err.message : String(err),
      };
    }
  }

  // Web Browser fallback simulation
  const trimmed = url.trim();
  if (!trimmed) {
    return { valid: false, url_type: "INVALID", is_playlist: false, video_id: null, playlist_id: null, message: "URL cannot be empty" };
  }

  if (trimmed.includes("playlist?list=")) {
    return {
      valid: true,
      url_type: "PLAYLIST",
      is_playlist: true,
      video_id: null,
      playlist_id: "PL_demo_playlist_123",
      message: "Valid YouTube Playlist detected",
    };
  }

  if (trimmed.includes("youtube.com/watch") || trimmed.includes("youtu.be/")) {
    return {
      valid: true,
      url_type: "VIDEO",
      is_playlist: false,
      video_id: "demo_video_456",
      playlist_id: null,
      message: "Valid YouTube Video detected",
    };
  }

  return {
    valid: false,
    url_type: "INVALID",
    is_playlist: false,
    video_id: null,
    playlist_id: null,
    message: "Please enter a valid YouTube video or playlist link",
  };
}

/**
 * Invoke Rust IPC command get_app_info.
 */
export async function invokeGetAppInfo(): Promise<AppInfo> {
  if (isTauriEnv()) {
    try {
      return await invoke<AppInfo>("get_app_info");
    } catch {
      return { name: "Siphonix", version: "0.1.0", platform: "desktop" };
    }
  }

  return { name: "Siphonix", version: "0.1.0 (Dev Web)", platform: "browser" };
}

/**
 * Open native directory picker using @tauri-apps/plugin-dialog.
 */
export async function pickFolder(defaultPath?: string): Promise<string | null> {
  if (isTauriEnv()) {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath,
        title: "Select Download Destination Folder",
      });
      if (typeof selected === "string") return selected;
      return null;
    } catch (err) {
      console.warn("Folder picker dialog failed:", err);
      return null;
    }
  }

  // Browser dev fallback
  const mockPath = prompt("Enter download folder path:", defaultPath || "~/Downloads/Siphonix");
  return mockPath || null;
}
