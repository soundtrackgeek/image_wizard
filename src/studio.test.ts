import { describe, expect, it } from "vitest";
import { dimensions, parsedSeed, progressPercent, RATIOS } from "./studio";
describe("generation settings", () => {
  it("keeps native presets and aligns smaller dimensions to sixteen pixels", () => {
    RATIOS.forEach((preset, index) => {
      expect(dimensions(index, "native")).toEqual({
        width: preset.width,
        height: preset.height,
      });
      const draft = dimensions(index, "draft");
      expect(draft.width % 16).toBe(0);
      expect(draft.height % 16).toBe(0);
      expect(Math.max(draft.width, draft.height)).toBe(512);
    });
  });
  it("rejects fractional and out-of-range seeds while preserving zero", () => {
    expect(parsedSeed("")).toBeNull();
    expect(parsedSeed("0")).toBe(0);
    expect(() => parsedSeed("1.4")).toThrow();
    expect(() => parsedSeed("2147483648")).toThrow();
  });
  it("does not invent percentages without a denominator", () => {
    expect(progressPercent(0, null)).toBeNull();
    expect(progressPercent(1, 0)).toBeNull();
    expect(progressPercent(5, 8)).toBe(63);
  });
});
