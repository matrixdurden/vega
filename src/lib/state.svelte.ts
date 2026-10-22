import { api, on, type GameVersion, type InstalledMod, type LogLine, type Profile, type Settings, type Stage } from "./api";
import { locale, t } from "./i18n.svelte";

export type View = "home" | "mods" | "settings";
type Toast = { id: number; kind: "info" | "error"; text: string };

const MAX_LOG_LINES = 2000;

export const app = $state({
  ready: false,
  view: "home" as View,
  settings: null as Settings | null,
  profiles: [] as Profile[],
  selectedId: null as string | null,
  mods: [] as InstalledMod[],
  versions: [] as GameVersion[],
  launch: { phase: "idle" as "idle" | "preparing" | "running", stage: "metadata" as Stage, done: 0, total: 0 },
  logs: [] as LogLine[],
  consoleOpen: false,
  newProfileOpen: false,
  toast: null as Toast | null,
});

export function selectedProfile(): Profile | null {
  return app.profiles.find((profile) => profile.id === app.selectedId) ?? null;
}

let toastTimer: ReturnType<typeof setTimeout>;
export function toast(text: string, kind: Toast["kind"] = "info") {
  clearTimeout(toastTimer);
  app.toast = { id: Date.now(), kind, text };
  toastTimer = setTimeout(() => (app.toast = null), kind === "error" ? 7000 : 3500);
}

export function errorText(err: unknown): string {
  return typeof err === "string" ? err : err instanceof Error ? err.message : String(err);
}

export async function init() {
  wireEvents();
  try {
    const [settings, profiles] = await Promise.all([api.getSettings(), api.listProfiles()]);
    app.settings = settings;
    if (settings.language) locale.lang = settings.language;
    app.profiles = profiles;
    app.ready = true;

    if (profiles.length === 0) await createFirstProfile();
    const remembered = app.profiles.find((p) => p.id === settings.selectedProfile);
    const recent = [...app.profiles].sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0];
    app.selectedId = (remembered ?? recent)?.id ?? null;
    await loadMods();
  } catch (err) {
    app.ready = true;
    toast(errorText(err), "error");
  }
  loadVersions();
}

async function createFirstProfile() {
  await loadVersions();
  const latest = app.versions.find((version) => version.fabric) ?? app.versions[0];
  if (!latest) return;
  const profile = await api.createProfile(`Fabric ${latest.id}`, latest.id, latest.fabric ? "fabric" : "vanilla");
  app.profiles = [profile];
}

export async function loadVersions() {
  if (app.versions.length) return;
  try {
    app.versions = await api.gameVersions();
  } catch (err) {
    toast(errorText(err), "error");
  }
}

export async function loadMods() {
  app.mods = app.selectedId ? await api.installedMods(app.selectedId) : [];
}

export async function selectProfile(id: string) {
  app.selectedId = id;
  updateSettings({ selectedProfile: id });
  await loadMods();
}

export async function createProfile(name: string, gameVersion: string, loader: Profile["loader"]) {
  const profile = await api.createProfile(name, gameVersion, loader);
  app.profiles.push(profile);
  await selectProfile(profile.id);
}

export async function deleteProfile(id: string) {
  try {
    await api.deleteProfile(id);
    app.profiles = app.profiles.filter((profile) => profile.id !== id);
    if (app.selectedId === id) await selectProfile(app.profiles[0]?.id ?? "");
  } catch (err) {
    toast(errorText(err), "error");
  }
}

let saveTimer: ReturnType<typeof setTimeout>;
/** Applies a settings change at once and saves it shortly after. */
export function updateSettings(patch: Partial<Settings>) {
  if (!app.settings) return;
  Object.assign(app.settings, patch);
  if (patch.language) locale.lang = patch.language;
  clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    try {
      await api.saveSettings($state.snapshot(app.settings!));
    } catch (err) {
      toast(errorText(err), "error");
    }
  }, 400);
}

export async function launch() {
  const id = app.selectedId;
  if (!id || app.launch.phase !== "idle") return;
  clearTimeout(saveTimer);
  await api.saveSettings($state.snapshot(app.settings!)).catch(() => {});
  app.launch = { phase: "preparing", stage: "metadata", done: 0, total: 1 };
  app.logs = [];
  try {
    await api.launch(id);
  } catch (err) {
    app.launch.phase = "idle";
    toast(errorText(err), "error");
  }
}

export function stopGame() {
  api.stopGame().catch((err) => toast(errorText(err), "error"));
}

function wireEvents() {
  on("launch-progress", (event) => {
    if (app.launch.phase === "preparing") Object.assign(app.launch, event);
  });
  on("game-started", () => {
    app.launch.phase = "running";
    const profile = app.profiles.find((p) => p.id === app.selectedId);
    if (profile) profile.lastPlayed = Math.floor(Date.now() / 1000);
  });
  on("game-log", (line) => {
    app.logs.push(line);
    if (app.logs.length > MAX_LOG_LINES) app.logs.splice(0, app.logs.length - MAX_LOG_LINES);
  });
  on("game-exited", (code) => {
    app.launch.phase = "idle";
    if (code !== null && code !== 0) {
      toast(t("error.crashed", { code }), "error");
      app.consoleOpen = true;
    }
  });
}
