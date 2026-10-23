import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { mockCall, mockListen } from "./mock";

export type Loader = "vanilla" | "fabric";
export type Stage = "metadata" | "libraries" | "assets" | "java" | "mods" | "starting";
export type ProfileFolder = "mods" | "resourcepacks" | "shaderpacks" | "screenshots" | "saves";

export interface Settings {
  memoryMb: number;
  selectedProfile: string | null;
  language: "tr" | "en" | null;
  hideOnLaunch: boolean;
}

export interface Profile {
  id: string;
  name: string;
  /** A pixel icon key or an uploaded data: URL. */
  icon: string;
  gameVersion: string;
  loader: Loader;
  loaderVersion: string | null;
  client: string | null;
  memoryMb: number | null;
  lastPlayed: number | null;
}

/** What creating or editing a profile sends. */
export interface ProfileDraft {
  name: string;
  icon: string;
  gameVersion: string;
  loader: Loader;
  memoryMb: number | null;
}

export interface Account {
  id: string;
  kind: "offline" | "microsoft";
  username: string;
  active: boolean;
}

export interface DeviceCode {
  userCode: string;
  verificationUri: string;
  expiresIn: number;
}

export interface ClientInfo {
  id: string;
  name: string;
  kind: "utility" | "performance";
  homepage: string;
  versions: string[];
}

export interface GameVersion {
  id: string;
  releaseTime: string;
  fabric: boolean;
}

export interface InstalledMod {
  projectId: string;
  versionId: string;
  versionNumber: string;
  title: string;
  iconUrl: string | null;
  filename: string;
  dependency: boolean;
  enabled: boolean;
}

/** A Modrinth search result, for mods and modpacks alike. */
export interface Project {
  projectId: string;
  slug: string;
  title: string;
  description: string;
  iconUrl: string | null;
  downloads: number;
  author: string;
}

export interface ModResult extends Project {
  installed: boolean;
}

export interface ProgressEvent {
  stage: Stage;
  done: number;
  total: number;
}

export interface LogLine {
  line: string;
  error: boolean;
}

export interface Events {
  "launch-progress": ProgressEvent;
  "install-progress": ProgressEvent;
  "game-started": null;
  "game-log": LogLine;
  "game-exited": number | null;
}

/** Outside Tauri (plain `npm run dev` in a browser) a mock backend is used. */
export const isTauri = "__TAURI_INTERNALS__" in window;

function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return isTauri ? invoke<T>(command, args) : mockCall<T>(command, args);
}

/** Sends a file's bytes as the raw request body, with string headers. */
async function upload<T>(command: string, file: File, headers: Record<string, string>): Promise<T> {
  const encoded = Object.fromEntries(Object.entries(headers).map(([k, v]) => [k, encodeURIComponent(v)]));
  if (!isTauri) return mockCall<T>(command, { ...headers, size: file.size });
  return invoke<T>(command, new Uint8Array(await file.arrayBuffer()), { headers: encoded });
}

export function on<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): Promise<UnlistenFn> {
  return isTauri ? listen<Events[K]>(event, (e) => handler(e.payload)) : mockListen(event, handler);
}

export function toDraft(profile: Profile): ProfileDraft {
  const { name, icon, gameVersion, loader, memoryMb } = profile;
  return { name, icon, gameVersion, loader, memoryMb };
}

export const api = {
  getSettings: () => call<Settings>("get_settings"),
  saveSettings: (settings: Settings) => call<Settings>("save_settings", { settings }),

  listAccounts: () => call<Account[]>("list_accounts"),
  addOfflineAccount: (username: string) => call<Account[]>("add_offline_account", { username }),
  setActiveAccount: (id: string) => call<Account[]>("set_active_account", { id }),
  removeAccount: (id: string) => call<Account[]>("remove_account", { id }),
  microsoftAvailable: () => call<boolean>("microsoft_available"),
  startMicrosoftLogin: () => call<DeviceCode>("start_microsoft_login"),
  finishMicrosoftLogin: () => call<Account[]>("finish_microsoft_login"),
  cancelMicrosoftLogin: () => call<void>("cancel_microsoft_login"),

  listProfiles: () => call<Profile[]>("list_profiles"),
  createProfile: (draft: ProfileDraft) => call<Profile>("create_profile", { draft }),
  updateProfile: (id: string, draft: ProfileDraft) => call<Profile>("update_profile", { id, draft }),
  deleteProfile: (id: string) => call<void>("delete_profile", { id }),
  gameVersions: () => call<GameVersion[]>("game_versions"),

  listClients: () => call<ClientInfo[]>("list_clients"),
  createClientProfile: (clientId: string, gameVersion: string, icon: string) =>
    call<Profile>("create_client_profile", { clientId, gameVersion, icon }),
  searchModpacks: (query: string) => call<Project[]>("search_modpacks", { query }),
  modpackVersions: (slug: string) => call<string[]>("modpack_versions", { slug }),
  createModpackProfile: (slug: string, gameVersion: string, name: string, icon: string) =>
    call<Profile>("create_modpack_profile", { slug, gameVersion, name, icon }),
  importModpack: (file: File, icon: string) => upload<Profile>("import_modpack", file, { "x-icon": icon }),

  installedMods: (profileId: string) => call<InstalledMod[]>("installed_mods", { profileId }),
  searchMods: (profileId: string, query: string) => call<ModResult[]>("search_mods", { profileId, query }),
  installMod: (profileId: string, projectId: string) => call<InstalledMod[]>("install_mod", { profileId, projectId }),
  addModFile: (profileId: string, file: File) =>
    upload<InstalledMod[]>("add_mod_file", file, { "x-profile-id": profileId, "x-file-name": file.name }),
  setModEnabled: (profileId: string, projectId: string, enabled: boolean) =>
    call<InstalledMod[]>("set_mod_enabled", { profileId, projectId, enabled }),
  removeMod: (profileId: string, projectId: string) => call<InstalledMod[]>("remove_mod", { profileId, projectId }),
  openProfileFolder: (profileId: string, folder?: ProfileFolder) =>
    call<void>("open_profile_folder", { profileId, folder: folder ?? null }),

  launch: (profileId: string) => call<void>("launch", { profileId }),
  stopGame: () => call<void>("stop_game"),
};
