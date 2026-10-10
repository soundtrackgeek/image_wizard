export const RATIOS = [
  { label: "1:1", name: "Square", width: 2048, height: 2048 },
  { label: "4:3", name: "Landscape", width: 2400, height: 1792 },
  { label: "3:4", name: "Portrait", width: 1792, height: 2400 },
  { label: "3:2", name: "Photo", width: 2528, height: 1696 },
  { label: "2:3", name: "Poster", width: 1696, height: 2528 },
  { label: "16:9", name: "Wide", width: 2752, height: 1536 },
  { label: "9:16", name: "Tall", width: 1536, height: 2752 },
];
export function dimensions(
  ratio: number,
  resolution: "draft" | "standard" | "native",
) {
  const preset = RATIOS[ratio] ?? RATIOS[0];
  const scale =
    resolution === "native"
      ? 1
      : resolution === "draft"
        ? 512 / Math.max(preset.width, preset.height)
        : 1024 / Math.max(preset.width, preset.height);
  return {
    width: Math.max(256, Math.round((preset.width * scale) / 16) * 16),
    height: Math.max(256, Math.round((preset.height * scale) / 16) * 16),
  };
}
export function parsedSeed(seed: string): number | null {
  if (!seed.trim()) return null;
  if (!/^\d+$/.test(seed))
    throw new Error("Seed must be a whole number between 0 and 2147483647.");
  const result = Number(seed);
  if (result > 2147483647)
    throw new Error("Seed must be between 0 and 2147483647.");
  return result;
}
export function progressPercent(
  completed: number | null,
  total: number | null,
): number | null {
  return completed !== null && total !== null && total > 0
    ? Math.min(100, Math.round((completed / total) * 100))
    : null;
}
export function bytes(value: number) {
  return value >= 1e9
    ? `${(value / 1e9).toFixed(1)} GB`
    : `${(value / 1e6).toFixed(0)} MB`;
}
