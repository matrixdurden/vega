import { api, on, type Account, type GameVersion, type InstalledMod, type LogLine, type Profile, type ProfileDraft, type Settings, type Stage } from "./api";
import { locale, t } from "./i18n.svelte";

export type View = "profile" | "settings";
export type ProfileTab = "mods" | "settings";
type Toast = { id: number; kind: "info" | "error"; text: string };

/** One game being prepared or played; several profiles can run at once. */
export type Run = { phase: "preparing" | "running"; stage: Stage; done: number; total: number };

const MAX_LOG_LINES = 2000;

export const app = $state({
  ready: false,
  view: "profile" as View,
  tab: "mods" as ProfileTab,
  settings: null as Settings | null,
  accounts: [] as Account[],
  profiles: [] as Profile[],
  selectedId: null as string | null,
  mods: [] as InstalledMod[],
  versions: [] as GameVersion[],
  runs: {} as Record<string, Run>,
  logs: {} as Record<string, LogLine[]>,
  /** Profile whose console is open. */
  console: null as string | null,
  modBrowserOpen: false,
  createOpen: false,
  accountsOpen: false,
  toast: null as Toast | null,
});

export function selectedProfile(): Profile | null {
  return app.profiles.find((profile) => profile.id === app.selectedId) ?? null;
}

export function activeAccount(): Account | null {
  return app.accounts.find((account) => account.active) ?? null;
}

let toastTimer: ReturnType<typeof setTimeout>;
export function toast(text: string, kind: Toast["kind"] = "info") {
  clearTimeout(toastTimer);
  app.toast = { id: Date.now(), kind, text };
  toastTimer = setTimeout(() => (app.toast = null), kind === "error" ? 7000 : 3000);
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
    const [settings, profiles, accounts] = await Promise.all([api.getSettings(), api.listProfiles(), api.listAccounts()]);
    app.settings = settings;
    if (settings.language) locale.lang = settings.language;
    app.profiles = profiles;
    app.accounts = accounts;
    app.ready = true;
    if (!accounts.length) app.accountsOpen = true;

    if (profiles.length === 0) await createFirstProfile();
    const remembered = app.profiles.find((p) => p.id === settings.selectedProfile);
    const recent = [...app.profiles].sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0];
    const first = remembered ?? recent;
    if (first) await selectProfile(first.id, false);
  } catch (err) {
    app.ready = true;
    toast(errorText(err), "error");
  }
}

async function createFirstProfile() {
  await loadVersions();
  const latest = app.versions.find((version) => version.fabric) ?? app.versions[0];
  if (!latest) return;
  const loader = latest.fabric ? "fabric" : "vanilla";
  const profile = await api.createProfile({ name: "Survival", icon: "grass", gameVersion: latest.id, loader, memoryMb: null });
  app.profiles = [profile];
}

/** Game versions are only fetched when a form needs them. */
export async function loadVersions() {
  if (app.versions.length) return;
  try {
    app.versions = await api.gameVersions();
  } catch (err) {
    toast(errorText(err), "error");
  }
}

export async function selectProfile(id: string, remember = true) {
  app.view = "profile";
  app.modBrowserOpen = false;
  if (app.selectedId === id) return;
  app.selectedId = id;
  app.mods = [];
  if (remember) updateSettings({ selectedProfile: id });
  const mods = await attempt(() => api.installedMods(id));
  if (mods && app.selectedId === id) app.mods = mods;
}

export async function addProfile(profile: Profile) {
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

export async function launch(id: string) {
  if (app.runs[id]) return;
  if (!activeAccount()) {
    app.accountsOpen = true;
    return;
  }
  clearTimeout(saveTimer);
  await api.saveSettings($state.snapshot(app.settings!)).catch(() => {});
  app.runs[id] = { phase: "preparing", stage: "metadata", done: 0, total: 1 };
  app.logs[id] = [];
  try {
    await api.launch(id);
  } catch (err) {
    delete app.runs[id];
    toast(errorText(err), "error");
  }
}

export function stopGame(id: string) {
  attempt(() => api.stopGame(id));
}

function wireEvents() {
  on("launch-progress", ({ profileId, ...progress }) => {
    const run = app.runs[profileId];
    if (run?.phase === "preparing") Object.assign(run, progress);
  });
  on("game-started", ({ profileId }) => {
    const run = app.runs[profileId];
    if (run) run.phase = "running";
    const profile = app.profiles.find((p) => p.id === profileId);
    if (profile) profile.lastPlayed = Math.floor(Date.now() / 1000);
  });
  on("game-log", ({ profileId, line, error }) => {
    const logs = (app.logs[profileId] ??= []);
    logs.push({ line, error });
    if (logs.length > MAX_LOG_LINES) logs.splice(0, logs.length - MAX_LOG_LINES);
  });
  on("game-exited", ({ profileId, code }) => {
    delete app.runs[profileId];
    if (code !== null && code !== 0) {
      toast(t("error.crashed", { code }), "error");
      app.console = profileId;
    }
  });
}
