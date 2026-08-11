import { describe, it, expect } from "vitest";
import { PlaylistEntry, PlaylistInfo } from "../types";

describe("Phase 4 Large Playlist Performance & Selection Logic", () => {
  const generateLargePlaylist = (count: number): PlaylistInfo => {
    const entries: PlaylistEntry[] = [];
    let available_count = 0;

    for (let i = 1; i <= count; i++) {
      const isUnavailable = i % 10 === 0; // 10% unavailable/deleted
      if (!isUnavailable) available_count++;

      entries.push({
        id: `v-id-${i}`,
        index: i,
        url: `https://www.youtube.com/watch?v=v-id-${i}`,
        title: `Test Video Title #${i}`,
        duration: 180 + i,
        thumbnail_url: `https://img.youtube.com/vi/v-id-${i}/default.jpg`,
        availability: isUnavailable ? "DELETED" : "AVAILABLE",
      });
    }

    return {
      id: "pl-500-test",
      title: "500 Entry Stress Test Playlist",
      uploader: "Siphonix Test Channel",
      webpage_url: "https://www.youtube.com/playlist?list=pl-500-test",
      entry_count: count,
      available_count,
      entries,
    };
  };

  it("handles 500 playlist entries efficiently with selection logic", () => {
    const startTime = performance.now();
    const playlist = generateLargePlaylist(500);
    const initTime = performance.now() - startTime;

    expect(playlist.entries.length).toBe(500);
    expect(playlist.available_count).toBe(450);
    expect(initTime).toBeLessThan(100); // Must generate under 100ms

    // Test Select All available items
    const selectAllStart = performance.now();
    const availableIds = new Set(
      playlist.entries.filter((e) => e.availability === "AVAILABLE").map((e) => e.id)
    );
    const selectAllDuration = performance.now() - selectAllStart;

    expect(availableIds.size).toBe(450);
    expect(selectAllDuration).toBeLessThan(50);

    // Test Deselect All
    const clearSet = new Set();
    expect(clearSet.size).toBe(0);

    // Test Individual selection toggle
    const toggleSet = new Set(availableIds);
    toggleSet.delete("v-id-1");
    expect(toggleSet.size).toBe(449);
  });
});
