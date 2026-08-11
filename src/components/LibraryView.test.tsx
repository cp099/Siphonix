import { describe, it, expect } from "vitest";
import { LibraryItem } from "../types";

describe("Library Performance & Filtering Suite", () => {
  it("filters and sorts 1,000 library items in under 10ms", () => {
    const mockItems: LibraryItem[] = Array.from({ length: 1000 }, (_, i) => ({
      id: `lib-${i}`,
      jobId: `job-${i}`,
      sourceVideoId: `video-${i}`,
      title: `Sample Video #${i} ${i % 2 === 0 ? "Blender Tutorial" : "Music Track"}`,
      filePath: `/downloads/Sample_Video_${i}.${i % 2 === 0 ? "mp4" : "mp3"}`,
      fileName: `Sample_Video_${i}.${i % 2 === 0 ? "mp4" : "mp3"}`,
      fileExtension: i % 2 === 0 ? "mp4" : "mp3",
      mediaMode: i % 2 === 0 ? "video" : "audio",
      format: i % 2 === 0 ? "MP4" : "MP3",
      quality: i % 2 === 0 ? "1080p" : "320k",
      fileSizeBytes: 1024 * 1024 * (i + 1),
      sourceUrl: `https://www.youtube.com/watch?v=video-${i}`,
      createdAt: new Date(Date.now() - i * 100000).toISOString(),
      completedAt: new Date(Date.now() - i * 50000).toISOString(),
      lastVerifiedAt: new Date().toISOString(),
      fileStatus: i % 10 === 0 ? "MISSING" : "AVAILABLE",
    }));

    const startTime = performance.now();

    // Perform search, filter, and sort in memory
    const searchStr = "Blender";
    const modeFilter: string = "video";
    const statusFilter: string = "AVAILABLE";

    const filtered = mockItems.filter((item) => {
      const matchSearch = item.title.toLowerCase().includes(searchStr.toLowerCase());
      const matchMode = modeFilter === "ALL" || item.mediaMode === modeFilter;
      const matchStatus = statusFilter === "ALL" || item.fileStatus === statusFilter;
      return matchSearch && matchMode && matchStatus;
    });

    filtered.sort((a, b) => new Date(b.completedAt).getTime() - new Date(a.completedAt).getTime());

    const duration = performance.now() - startTime;

    expect(filtered.length).toBeGreaterThan(0);
    expect(duration).toBeLessThan(10); // Less than 10 milliseconds
  });
});
