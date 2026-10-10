import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { GenerateRequest, Reference, Snapshot } from "./types";
export const desktop = isTauri();
const browserSnapshot: Snapshot = {
  preferences: {
    modelDir: "Choose a location in the desktop app",
    profile: "nf4",
    licenseAccepted: false,
    checkedProfile: null,
  },
  hardware: {
    gpuName: null,
    vramMb: null,
    ramBytes: 0,
    freeBytes: 0,
    modelBytes: 32493327348,
  },
  modelDownloaded: false,
  runtimeInstalled: false,
  ready: false,
  operation: null,
  artworks: [],
  dataDir: "",
};
function requireDesktop() {
  if (!desktop)
    throw new Error(
      "Model installation and generation run in the Windows desktop app. Start it with npm run desktop.",
    );
}
export async function snapshot() {
  return desktop
    ? invoke<Snapshot>("get_snapshot")
    : structuredClone(browserSnapshot);
}
export const imageUrl = (path: string) =>
  desktop ? convertFileSrc(path) : path;
export async function preferences(
  profile: string,
  licenseAccepted: boolean,
  directory: string | null = null,
) {
  requireDesktop();
  await invoke("save_preferences", { profile, directory, licenseAccepted });
}
export async function install() {
  requireDesktop();
  return invoke<string>("start_install");
}
export async function check() {
  requireDesktop();
  return invoke<string>("check_engine");
}
export async function generate(request: GenerateRequest) {
  requireDesktop();
  return invoke<string>("start_generation", { request });
}
export async function cancel() {
  requireDesktop();
  await invoke("cancel_operation");
}
export async function pickDirectory() {
  requireDesktop();
  return open({
    directory: true,
    multiple: false,
    title: "Choose a drive or folder for your model",
  });
}
export async function importImages(): Promise<Reference[]> {
  requireDesktop();
  const paths = await open({
    multiple: true,
    title: "Add reference images",
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }],
  });
  if (!paths) return [];
  const result: Reference[] = [];
  for (const path of Array.isArray(paths) ? paths : [paths])
    result.push(await invoke<Reference>("import_reference", { path }));
  return result;
}
export async function importPath(path: string) {
  requireDesktop();
  return invoke<Reference>("import_reference", { path });
}
export async function exportImage(id: string) {
  requireDesktop();
  const destination = await save({
    title: "Export your image",
    defaultPath: `image-wizard-${id.slice(0, 8)}.png`,
    filters: [{ name: "PNG image", extensions: ["png"] }],
  });
  if (destination) await invoke("export_image", { id, destination });
}
export async function license() {
  const url = "https://github.com/QwenLM/Qwen-Image-2.1/blob/main/LICENSE";
  if (desktop) await openUrl(url);
  else window.open(url, "_blank", "noopener,noreferrer");
}
