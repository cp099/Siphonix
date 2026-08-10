import { describe, it, expect } from "vitest";
import { UrlInputSchema } from "./index";

describe("UrlInputSchema validation", () => {
  it("validates valid YouTube video URLs", () => {
    const validVideoUrl = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    const result = UrlInputSchema.safeParse(validVideoUrl);
    expect(result.success).toBe(true);
  });

  it("validates valid YouTube playlist URLs", () => {
    const validPlaylistUrl = "https://www.youtube.com/playlist?list=PL3rVcngGfeeqE5H9N9-9Q5yJ4-3gM2f2j";
    const result = UrlInputSchema.safeParse(validPlaylistUrl);
    expect(result.success).toBe(true);
  });

  it("rejects empty or non-YouTube URLs", () => {
    expect(UrlInputSchema.safeParse("").success).toBe(false);
    expect(UrlInputSchema.safeParse("https://vimeo.com/123456").success).toBe(false);
  });
});
