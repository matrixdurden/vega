// In-memory backend for working on the UI in a plain browser (`npm run dev`).
// Nothing here runs inside the real app.

import type { Events, GameVersion, InstalledMod, ModResult, Profile, Settings } from "./api";

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

let settings: Settings = { username: "Steve", memoryMb: 4096, selectedProfile: null, language: null, hideOnLaunch: true };
const profiles: Profile[] = [];
const mods = new Map<string, InstalledMod[]>();
let running: ReturnType<typeof setTimeout> | null = null;

const versions: GameVersion[] = ["1.21.10", "1.21.8", "1.21.5", "1.21.4", "1.21.1", "1.20.6", "1.20.1", "1.19.4", "1.18.2", "1.16.5", "1.12.2", "1.8.9"].map(
  (id, i) => ({ id, releaseTime: "", fabric: i < 10 }),
);

const catalog: Omit<ModResult, "installed">[] = [
  ["AANobbMI", "sodium", "Sodium", "A modern rendering engine that greatly improves frame rates and stutter.", 72_000_000, "jellysquid3"],
  ["YL57xq9U", "iris", "Iris Shaders", "Shader pack support that works with Sodium.", 48_000_000, "coderbot"],
  ["P7dR8mSH", "fabric-api", "Fabric API", "Core library for most Fabric mods.", 110_000_000, "modmuss50"],
  ["gvQqBUqZ", "lithium", "Lithium", "Optimises game physics, mob AI and block ticking without changing gameplay.", 41_000_000, "jellysquid3"],
  ["1bokaNcj", "xaeros-minimap", "Xaero's Minimap", "A clean minimap that keeps the vanilla feel.", 30_000_000, "thexaero"],
  ["mOgUt4GM", "modmenu", "Mod Menu", "Adds a mod menu to view the list of mods you have installed.", 65_000_000, "Prospector"],
].map(([projectId, slug, title, description, downloads, author]) => ({
  projectId: projectId as string,
  slug: slug as string,
  title: title as string,
  description: description as string,
  downloads: downloads as number,
  author: author as string,
  iconUrl: null,
}));

export async function mockCall<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  await sleep(120);
  const profileId = args.profileId as string;
  const result = ((): unknown => {
    switch (command) {
      case "get_settings":
        return settings;
      case "save_settings":
        settings = args.settings as Settings;
        if (!/^\w{3,16}$/.test(settings.username)) throw "username must be 3-16 letters, digits or underscores";
        return settings;
      case "list_profiles":
        return profiles;
      case "create_profile": {
        const name = (args.name as string) || (args.gameVersion as string);
        const profile: Profile = {
          id: name.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
          name,
          gameVersion: args.gameVersion as string,
          loader: args.loader as Profile["loader"],
          loaderVersion: args.loader === "fabric" ? "0.17.2" : null,
          lastPlayed: null,
        };
        profiles.push(profile);
        return profile;
      }
      case "delete_profile":
        profiles.splice(profiles.findIndex((p) => p.id === args.id), 1);
        return null;
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
            list.push({ projectId: id, versionId: "", versionNumber: "1.0.0", title: mod.title, iconUrl: null, filename: `${mod.slug}.jar`, dependency });
        };
        add(args.projectId as string, false);
        if (args.projectId !== "AANobbMI") add("P7dR8mSH", true);
        mods.set(profileId, list);
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
        return simulateLaunch();
      case "stop_game":
        if (running) clearTimeout(running);
        running = null;
        emit("game-exited", 0);
        return null;
      default:
        throw `unknown command ${command}`;
    }
  })();
  return (await result) as T;
}

async function simulateLaunch() {
  for (const [stage, total] of [["metadata", 1], ["libraries", 64], ["assets", 3800], ["java", 420], ["starting", 1]] as const) {
    for (let step = 0; step <= 10; step++) {
      emit("launch-progress", { stage, done: Math.round((total * step) / 10), total });
      await sleep(70);
    }
  }
  emit("game-started", null);
  ["[main/INFO]: Loading Minecraft 1.21.10 with Fabric Loader", "[Render thread/INFO]: Backend library: LWJGL", "[Render thread/WARN]: Missing sound for event"].forEach(
    (line, i) => setTimeout(() => emit("game-log", { line, error: line.includes("WARN") }), 300 * (i + 1)),
  );
  running = setTimeout(() => emit("game-exited", 0), 15_000);
}
