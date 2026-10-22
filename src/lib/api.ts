import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { mockCall, mockListen } from "./mock";

export type Loader = "vanilla" | "fabric";
export type Stage = "metadata" | "libraries" | "assets" | "java" | "mods" | "starting";

export interface Settings {
  username: string;
  memoryMb: number;
  selectedProfile: string | null;
  language: "tr" | "en" | null;
  hideOnLaunch: boolean;
}

export interface Profile {
  id: string;
  name: string;
  gameVersion: string;
  loader: Loader;
  loaderVersion: string | null;
  lastPlayed: number | null;
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
}

export interface ModResult {
  projectId: string;
  slug: string;
  title: string;
  description: string;
  iconUrl: string | null;
  downloads: number;
  author: string;
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
  "game-started": null;
  "game-log": LogLine;
  "game-exited": number | null;
}

/** Outside Tauri (plain `npm run dev` in a browser) a mock backend is used. */
export const isTauri = "__TAURI_INTERNALS__" in window;

function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return isTauri ? invoke<T>(command, args) : mockCall<T>(command, args);
}

export function on<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): Promise<UnlistenFn> {
  return isTauri ? listen<Events[K]>(event, (e) => handler(e.payload)) : mockListen(event, handler);
}

export const api = {
  getSettings: () => call<Settings>("get_settings"),
  saveSettings: (settings: Settings) => call<Settings>("save_settings", { settings }),
  listProfiles: () => call<Profile[]>("list_profiles"),
  createProfile: (name: string, gameVersion: string, loader: Loader) =>
    call<Profile>("create_profile", { name, gameVersion, loader }),
  deleteProfile: (id: string) => call<void>("delete_profile", { id }),
  gameVersions: () => call<GameVersion[]>("game_versions"),
  installedMods: (profileId: string) => call<InstalledMod[]>("installed_mods", { profileId }),
  searchMods: (profileId: string, query: string) => call<ModResult[]>("search_mods", { profileId, query }),
  installMod: (profileId: string, projectId: string) => call<InstalledMod[]>("install_mod", { profileId, projectId }),
  removeMod: (profileId: string, projectId: string) => call<InstalledMod[]>("remove_mod", { profileId, projectId }),
  openProfileFolder: (profileId: string) => call<void>("open_profile_folder", { profileId }),
  launch: (profileId: string) => call<void>("launch", { profileId }),
  stopGame: () => call<void>("stop_game"),
};
