// In-memory backend for working on the UI in a plain browser (`npm run dev`).
// Nothing here runs inside the real app.

import type { Account, ClientInfo, Events, GameVersion, InstalledMod, ModResult, Profile, ProfileDraft, Settings } from "./api";

type Handler = (payload: never) => void;
const handlers = new Map<string, Set<Handler>>();

function emit<K extends keyof Events>(event: K, payload: Events[K]) {
  handlers.get(event)?.forEach((handler) => (handler as (p: Events[K]) => void)(payload));
}

export async function mockListen<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void) {
  const set = handlers.get(event) ?? new Set();
  set.add(handler as Handler);
  handlers.set(event, set);
  return () => set.delete(handler as Handler);
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const now = () => Math.floor(Date.now() / 1000);

let settings: Settings = { memoryMb: 4096, selectedProfile: null, language: null, hideOnLaunch: true };
let accounts: Account[] = [];
const profiles: Profile[] = [];
const mods = new Map<string, InstalledMod[]>();
let running: ReturnType<typeof setTimeout> | null = null;

const versions: GameVersion[] = ["26.3", "26.2", "26.1", "1.21.11", "1.21.8", "1.21.4", "1.21.1", "1.20.1", "1.16.5", "1.12.2", "1.8.9"].map(
  (id, i) => ({ id, releaseTime: "", fabric: i < 9 }),
);

const clients: ClientInfo[] = [
  { id: "meteor", name: "Meteor Client", kind: "utility", homepage: "https://meteorclient.com", versions: ["26.3", "26.2", "1.21.11", "1.21.8"] },
  { id: "fabulously-optimized", name: "Fabulously Optimized", kind: "performance", homepage: "https://modrinth.com/modpack/fabulously-optimized", versions: ["26.3", "26.2", "1.21.11"] },
];

const catalog: Omit<ModResult, "installed">[] = [
  ["AANobbMI", "sodium", "Sodium", "A modern rendering engine that greatly improves frame rates and stutter.", 252_000_000, "jellysquid3"],
  ["YL57xq9U", "iris", "Iris Shaders", "Shader pack support that works with Sodium.", 98_000_000, "coderbot"],
  ["P7dR8mSH", "fabric-api", "Fabric API", "Core library for most Fabric mods.", 210_000_000, "modmuss50"],
  ["gvQqBUqZ", "lithium", "Lithium", "Optimises game physics, mob AI and block ticking without changing gameplay.", 81_000_000, "jellysquid3"],
  ["1bokaNcj", "xaeros-minimap", "Xaero's Minimap", "A clean minimap that keeps the vanilla feel.", 60_000_000, "thexaero"],
].map(([projectId, slug, title, description, downloads, author]) => ({
  projectId: projectId as string,
  slug: slug as string,
  title: title as string,
  description: description as string,
  downloads: downloads as number,
  author: author as string,
  iconUrl: null,
}));

function slug(name: string) {
  const base = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "profile";
  let id = base;
  for (let n = 2; profiles.some((p) => p.id === id); n++) id = `${base}-${n}`;
  return id;
}

function newProfile(draft: ProfileDraft, client: string | null = null): Profile {
  const profile: Profile = {
    ...draft,
    id: slug(draft.name || draft.gameVersion),
    name: draft.name.trim() || draft.gameVersion,
    loaderVersion: draft.loader === "fabric" ? "0.19.5" : null,
    client,
    lastPlayed: null,
  };
  profiles.push(profile);
  return profile;
}

function activate(id: string) {
  accounts = accounts.map((a) => ({ ...a, active: a.id === id }));
  return accounts;
}

export async function mockCall<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  await sleep(100);
  const profileId = args.profileId as string;
  const result = ((): unknown => {
    switch (command) {
      case "get_settings":
        return settings;
      case "save_settings":
        return (settings = args.settings as Settings);
      case "list_accounts":
        return accounts;
      case "add_offline_account": {
        const username = (args.username as string).trim();
        if (!/^\w{3,16}$/.test(username)) throw "username must be 3-16 letters, digits or underscores";
        const id = `offline-${username}`;
        if (!accounts.some((a) => a.id === id)) accounts.push({ id, kind: "offline", username, active: false });
        return activate(id);
      }
      case "set_active_account":
        return activate(args.id as string);
      case "remove_account":
        accounts = accounts.filter((a) => a.id !== args.id);
        if (accounts.length && !accounts.some((a) => a.active)) accounts[0].active = true;
        return accounts;
      case "microsoft_available":
        return false;
      case "list_profiles":
        return profiles;
      case "create_profile":
        return newProfile(args.draft as ProfileDraft);
      case "update_profile": {
        const profile = profiles.find((p) => p.id === args.id)!;
        const draft = args.draft as ProfileDraft;
        if (profile.gameVersion !== draft.gameVersion || profile.loader !== draft.loader)
          profile.loaderVersion = draft.loader === "fabric" ? "0.19.5" : null;
        Object.assign(profile, draft, { name: draft.name.trim() || draft.gameVersion });
        return profile;
      }
      case "delete_profile":
        profiles.splice(profiles.findIndex((p) => p.id === args.id), 1);
        return null;
      case "list_clients":
        return clients;
      case "create_client_profile":
        return simulateClientInstall(args.clientId as string, args.gameVersion as string, args.icon as string);
      case "game_versions":
        return versions;
      case "installed_mods":
        return mods.get(profileId) ?? [];
      case "search_mods": {
        const query = (args.query as string).toLowerCase();
        const installed = mods.get(profileId) ?? [];
        return catalog
          .filter((mod) => mod.title.toLowerCase().includes(query))
          .map((mod) => ({ ...mod, installed: installed.some((m) => m.projectId === mod.projectId) }));
      }
      case "install_mod": {
        const list = mods.get(profileId) ?? [];
        const add = (id: string, dependency: boolean) => {
          const mod = catalog.find((m) => m.projectId === id)!;
          if (!list.some((m) => m.projectId === id))
            list.push({ projectId: id, versionId: "", versionNumber: "1.0.0", title: mod.title, iconUrl: null, filename: `${mod.slug}.jar`, dependency, enabled: true });
        };
        add(args.projectId as string, false);
        if (args.projectId !== "AANobbMI") add("P7dR8mSH", true);
        mods.set(profileId, list);
        return list;
      }
      case "set_mod_enabled": {
        const list = mods.get(profileId) ?? [];
        const mod = list.find((m) => m.projectId === args.projectId);
        if (mod) mod.enabled = args.enabled as boolean;
        return list;
      }
      case "remove_mod": {
        const list = (mods.get(profileId) ?? []).filter((m) => m.projectId !== args.projectId);
        mods.set(profileId, list);
        return list;
      }
      case "open_profile_folder":
        return null;
      case "launch":
        if (!accounts.length) throw "add an account before playing";
        return simulateLaunch(profileId);
      case "stop_game":
        if (running) clearTimeout(running);
        running = null;
        emit("game-exited", 0);
        return null;
      default:
        throw `unknown command ${command}`;
    }
  })();
  return structuredClone(await result) as T;
}

async function simulateClientInstall(clientId: string, gameVersion: string, icon: string) {
  const client = clients.find((c) => c.id === clientId)!;
  for (let step = 0; step <= 10; step++) {
    emit("install-progress", { stage: "mods", done: step * 5, total: 50 });
    await sleep(80);
  }
  const profile = newProfile({ name: client.name, icon, gameVersion, loader: "fabric", memoryMb: null }, clientId);
  mods.set(profile.id, [
    { projectId: clientId, versionId: "", versionNumber: gameVersion, title: client.name, iconUrl: null, filename: `${clientId}.jar`, dependency: false, enabled: true },
  ]);
  return profile;
}

async function simulateLaunch(profileId: string) {
  for (const [stage, total] of [["metadata", 1], ["libraries", 64], ["assets", 3800], ["java", 420], ["starting", 1]] as const) {
    for (let step = 0; step <= 10; step++) {
      emit("launch-progress", { stage, done: Math.round((total * step) / 10), total });
      await sleep(50);
    }
  }
  const profile = profiles.find((p) => p.id === profileId);
  if (profile) profile.lastPlayed = now();
  emit("game-started", null);
  ["[main/INFO]: Loading Minecraft with Fabric Loader 0.19.5", "[Render thread/INFO]: Backend library: LWJGL"].forEach((line, i) =>
    setTimeout(() => emit("game-log", { line, error: false }), 300 * (i + 1)),
  );
  running = setTimeout(() => emit("game-exited", 0), 8_000);
}
