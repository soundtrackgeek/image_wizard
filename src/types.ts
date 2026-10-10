export type Mode = "create" | "transform" | "edit";
export type View = Mode | "library" | "models";
export interface Preferences {
  modelDir: string;
  profile: "nf4" | "bf16";
  licenseAccepted: boolean;
  checkedProfile: string | null;
}
export interface Operation {
  id: string;
  kind: string;
  phase: string;
  detail: string;
  completed: number | null;
  total: number | null;
  startedAt: number;
  finishedAt: number | null;
}
export interface Artwork {
  id: string;
  path: string;
  prompt: string;
  effectivePrompt: string;
  mode: Mode;
  width: number;
  height: number;
  seed: number;
  hasAlpha: boolean;
  transparencyRequested: boolean;
  profile: string;
  createdAt: number;
  references: string[];
  parentId: string | null;
  provenance?: Record<string, unknown> | null;
}
export interface Reference {
  id: string;
  path: string;
  name: string;
  width: number;
  height: number;
}
export interface Snapshot {
  preferences: Preferences;
  hardware: {
    gpuName: string | null;
    vramMb: number | null;
    ramBytes: number;
    freeBytes: number;
    modelBytes: number;
  };
  modelDownloaded: boolean;
  runtimeInstalled: boolean;
  ready: boolean;
  operation: Operation | null;
  artworks: Artwork[];
  dataDir: string;
}
export interface GenerateRequest {
  prompt: string;
  mode: Mode;
  width: number;
  height: number;
  seed: number | null;
  transparent: boolean;
  references: string[];
  parentId: string | null;
}
