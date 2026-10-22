import {
  api,
  on,
  type GameVersion,
  type InstalledMod,
  type LogLine,
  type Profile,
  type ProfileDraft,
  type Settings,
  type Stage,
  type World,
} from "./api";
import { locale, t } from "./i18n.svelte";

export type View = "profile" | "settings";
export type ProfileTab = "mods" | "worlds" | "settings";
type Toast = { id: number; kind: "info" | "error"; text: string };
type Dialog = { mode: "create" } | { mode: "edit"; id: string };

const MAX_LOG_LINES = 2000;

export const app = $state({
  ready: false,
  view: "profile" as View,
  tab: "mods" as ProfileTab,
  settings: null as Settings | null,
  profiles: [] as Profile[],
  selectedId: null as string | null,
  mods: [] as InstalledMod[],
  worlds: [] as World[],
  versions: [] as GameVersion[],
  launch: { phase: "idle" as "idle" | "preparing" | "running", stage: "metadata" as Stage, done: 0, total: 0 },
  logs: [] as LogLine[],
  consoleOpen: false,
  modBrowserOpen: false,
  dialog: null as Dialog | null,
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

/** Runs an action and shows its error as a toast instead of throwing. */
export async function attempt<T>(action: () => Promise<T>): Promise<T | undefined> {
  try {
    return await action();
  } catch (err) {
    toast(errorText(err), "error");
  }
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
    const first = remembered ?? recent;
    if (first) await selectProfile(first.id, false);
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
  const loader = latest.fabric ? "fabric" : "vanilla";
  const profile = await api.createProfile({ name: "Survival", icon: "grass", gameVersion: latest.id, loader, memoryMb: null });
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

async function loadProfileData(id: string) {
  const [mods, worlds] = await Promise.all([api.installedMods(id), api.listWorlds(id)]);
  if (app.selectedId === id) {
    app.mods = mods;
    app.worlds = worlds;
  }
}

export async function selectProfile(id: string, remember = true) {
  app.view = "profile";
  app.modBrowserOpen = false;
  if (app.selectedId === id) return;
  app.selectedId = id;
  app.mods = [];
  app.worlds = [];
  if (remember) updateSettings({ selectedProfile: id });
  await attempt(() => loadProfileData(id));
}

export async function createProfile(draft: ProfileDraft) {
  const profile = await api.createProfile(draft);
  app.profiles.push(profile);
  app.tab = "mods";
  await selectProfile(profile.id);
}

export async function updateProfile(id: string, draft: ProfileDraft) {
  const updated = await api.updateProfile(id, draft);
  const index = app.profiles.findIndex((profile) => profile.id === id);
  if (index >= 0) app.profiles[index] = updated;
}

export async function deleteProfile(id: string) {
  await attempt(async () => {
    await api.deleteProfile(id);
    app.profiles = app.profiles.filter((profile) => profile.id !== id);
    if (app.selectedId === id) {
      app.selectedId = null;
      const next = app.profiles[0];
      if (next) await selectProfile(next.id);
    }
  });
}

let saveTimer: ReturnType<typeof setTimeout>;
/** Applies a settings change at once and saves it shortly after. */
export function updateSettings(patch: Partial<Settings>) {
  if (!app.settings) return;
  Object.assign(app.settings, patch);
  if (patch.language) locale.lang = patch.language;
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => attempt(() => api.saveSettings($state.snapshot(app.settings!))), 400);
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
  attempt(() => api.stopGame());
}

function wireEvents() {
  on("launch-progress", (event) => {
    if (app.launch.phase === "preparing") Object.assign(app.launch, event);
  });
  on("game-started", () => {
    app.launch.phase = "running";
    const profile = selectedProfile();
    if (profile) profile.lastPlayed = Math.floor(Date.now() / 1000);
  });
  on("game-log", (line) => {
    app.logs.push(line);
    if (app.logs.length > MAX_LOG_LINES) app.logs.splice(0, app.logs.length - MAX_LOG_LINES);
  });
  on("game-exited", (code) => {
    app.launch.phase = "idle";
    if (app.selectedId) attempt(() => loadProfileData(app.selectedId!));
    if (code !== null && code !== 0) {
      toast(t("error.crashed", { code }), "error");
      app.consoleOpen = true;
    }
  });
}
