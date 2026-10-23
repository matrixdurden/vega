<script lang="ts">
  import { api, on, type ClientInfo, type Loader, type Profile, type Project } from "../api";
  import { compactNumber, t } from "../i18n.svelte";
  import { pixelIconKeys } from "../pixelIcons";
  import { addProfile, app, attempt, loadVersions } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ModIcon from "./ModIcon.svelte";
  import Modal from "./Modal.svelte";
  import ProfileFields from "./ProfileFields.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  type Mode = "blank" | "modpack" | "file";
  const CLIENT_ICONS: Record<string, string> = { meteor: "sword", "fabulously-optimized": "diamond" };

  let mode = $state<Mode>("blank");
  let busy = $state(false);
  let progress = $state(0);

  // Blank profile, also used for "profile from a jar"
  let icon = $state(pixelIconKeys[app.profiles.length % pixelIconKeys.length]);
  let name = $state("");
  let loader = $state<Loader>("fabric");
  let version = $state("");

  // Modpack search
  let query = $state("");
  let packs = $state<Project[]>([]);
  let searching = $state(false);
  let pack = $state<Project | null>(null);
  let packVersions = $state<string[]>([]);
  let packVersion = $state("");

  // From file
  let jars = $state<File[]>([]);
  let jarVersion = $state("");
  let clients = $state<ClientInfo[]>([]);
  let jarInput = $state<HTMLInputElement>();
  let packInput = $state<HTMLInputElement>();

  $effect(() => {
    const off = on("install-progress", (e) => (progress = e.total ? Math.round((e.done / e.total) * 100) : 0));
    return () => void off.then((unlisten) => unlisten());
  });

  $effect(() => {
    if (mode !== "modpack") return;
    const q = query;
    searching = true;
    const timer = setTimeout(async () => {
      const hits = await attempt(() => api.searchModpacks(q));
      if (q === query) {
        packs = hits ?? [];
        searching = false;
      }
    }, q ? 300 : 0);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    if (mode !== "file") return;
    loadVersions();
    if (!clients.length) attempt(() => api.listClients()).then((list) => (clients = list ?? []));
  });

  $effect(() => {
    if (!jarVersion) jarVersion = app.versions.find((v) => v.fabric)?.id ?? "";
  });

  async function pickPack(p: Project) {
    pack = p;
    packVersions = [];
    packVersion = "";
    packVersions = (await attempt(() => api.modpackVersions(p.slug))) ?? [];
    packVersion = packVersions[0] ?? "";
  }

  const close = () => !busy && (app.createOpen = false);

  async function run(action: () => Promise<Profile>) {
    if (busy) return;
    busy = true;
    progress = 0;
    const profile = await attempt(action);
    busy = false;
    if (profile) {
      app.createOpen = false;
      await addProfile(profile);
    }
  }

  const createBlank = () =>
    run(() => api.createProfile({ name: name.trim() || `${loader === "fabric" ? "Fabric" : "Vanilla"} ${version}`, icon, gameVersion: version, loader, memoryMb: null }));

  const installPack = () => pack && run(() => api.createModpackProfile(pack!.slug, packVersion, pack!.title, "chest"));

  async function importPack(file: File | undefined) {
    if (file) await run(() => api.importModpack(file, "chest"));
    if (packInput) packInput.value = "";
  }

  // A Fabric profile named after the first jar, with every picked jar added.
  const createFromJars = () =>
    run(async () => {
      const profile = await api.createProfile({
        name: name.trim() || jars[0].name.replace(/\.jar$/i, ""),
        icon,
        gameVersion: jarVersion,
        loader: "fabric",
        memoryMb: null,
      });
      for (const jar of jars) await api.addModFile(profile.id, jar);
      return profile;
    });
</script>

<Modal title={t("create.title")} width={600} onclose={close}>
  <div class="segmented">
    <button class:on={mode === "blank"} onclick={() => (mode = "blank")} disabled={busy}>{t("create.blank")}</button>
    <button class:on={mode === "modpack"} onclick={() => (mode = "modpack")} disabled={busy}>{t("create.modpack")}</button>
    <button class:on={mode === "file"} onclick={() => (mode = "file")} disabled={busy}>{t("create.file")}</button>
  </div>

  {#if mode === "blank"}
    <ProfileFields bind:icon bind:name bind:loader bind:version placeholder={`${loader === "fabric" ? "Fabric" : "Vanilla"} ${version}`} />
    <div class="actions">
      <button class="btn" onclick={close}>{t("create.cancel")}</button>
      <button class="btn green" onclick={createBlank} disabled={busy || !version}>
        {#if busy}<Icon name="loader" size={15} class="spin" />{/if}{t("create.button")}
      </button>
    </div>
  {:else if mode === "modpack"}
    <label class="search">
      <Icon name={searching ? "loader" : "search"} size={17} class={searching ? "spin" : ""} />
      <input class="input" bind:value={query} placeholder={t("create.searchModpacks")} spellcheck="false" />
    </label>
    <div class="packs">
      {#each packs as p (p.projectId)}
        <button class="pack" class:on={pack?.projectId === p.projectId} onclick={() => pickPack(p)} disabled={busy}>
          <ModIcon url={p.iconUrl} title={p.title} size={40} />
          <span class="text">
            <b class="mc">{p.title}</b>
            <span class="hint clamp">{p.description}</span>
          </span>
          <span class="hint nowrap"><Icon name="download" size={13} /> {compactNumber(p.downloads)}</span>
        </button>
      {:else}
        {#if !searching}<p class="hint empty">{t("create.noResults")}</p>{/if}
      {/each}
    </div>
    {#if pack}
      <div class="bottom">
        <select class="input version" bind:value={packVersion} disabled={busy || !packVersions.length} aria-label={t("form.version")}>
          {#if !packVersions.length}<option value="">{t("create.pickVersion")}</option>{/if}
          {#each packVersions as v (v)}<option value={v}>{v}</option>{/each}
        </select>
        {#if busy}<div class="bar"><div style:width="{progress}%"></div></div>{:else}<span class="grow"></span>{/if}
        <button class="btn green" onclick={installPack} disabled={busy || !packVersion}>
          {#if busy}<Icon name="loader" size={15} class="spin" />{/if}{busy ? t("create.installing") : t("create.install")}
        </button>
      </div>
    {/if}
  {:else}
    <section class="card">
      <h3>{t("create.jar")}</h3>
      <p class="hint">{t("create.jarHint")}</p>
      <div class="bottom">
        <button class="btn" onclick={() => jarInput?.click()} disabled={busy}><Icon name="upload" size={15} />{t("create.pickJar")}</button>
        <span class="hint grow">{jars.length ? (jars.length === 1 ? jars[0].name : t("create.files", { n: jars.length })) : ""}</span>
        <select class="input version" bind:value={jarVersion} aria-label={t("form.version")} disabled={busy}>
          {#each app.versions.filter((v) => v.fabric) as v (v.id)}<option value={v.id}>{v.id}</option>{/each}
        </select>
        <button class="btn green" onclick={createFromJars} disabled={busy || !jars.length || !jarVersion}>{t("create.button")}</button>
      </div>
      <input bind:this={jarInput} type="file" accept=".jar" multiple hidden onchange={(e) => (jars = [...(e.currentTarget.files ?? [])])} />
    </section>

    <section class="card">
      <h3>{t("create.mrpack")}</h3>
      <p class="hint">{t("create.mrpackHint")}</p>
      <div class="bottom">
        <button class="btn" onclick={() => packInput?.click()} disabled={busy}><Icon name="upload" size={15} />{t("create.pickPack")}</button>
        {#if busy}<div class="bar"><div style:width="{progress}%"></div></div>{/if}
      </div>
      <input bind:this={packInput} type="file" accept=".mrpack,.zip" hidden onchange={(e) => importPack(e.currentTarget.files?.[0])} />
    </section>

    {#if clients.length}
      <section class="card">
        <h3>{t("create.quick")}</h3>
        {#each clients as c (c.id)}
          <div class="client">
            <ProfileIcon icon={CLIENT_ICONS[c.id] ?? "sword"} size={34} />
            <span class="text">
              <b class="mc">{c.name}</b>
              {#if c.kind === "utility"}<span class="warn">{t("create.utilityWarning")}</span>{/if}
            </span>
            <button
              class="btn"
              disabled={busy || !c.versions.length}
              onclick={() => run(() => api.createClientProfile(c.id, c.versions[0], CLIENT_ICONS[c.id] ?? "sword"))}
            >
              {t("create.install")} · {c.versions[0] ?? "-"}
            </button>
          </div>
        {/each}
      </section>
    {/if}
  {/if}
</Modal>

<style>
  .actions,
  .bottom {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .grow {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .search {
    position: relative;
    display: block;
    color: var(--text-3);
  }

  .search :global(svg) {
    position: absolute;
    z-index: 1;
    left: 12px;
    top: 11px;
  }

  .search .input {
    padding-left: 38px;
  }

  .packs {
    height: 280px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .pack {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: 4px;
    background: var(--panel-2);
    border: 1px solid transparent;
    text-align: left;
  }

  .pack:hover:not(:disabled) {
    border-color: var(--border-strong);
  }

  .pack.on {
    border-color: var(--green-text);
    background: var(--select);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .text b {
    font-size: 13.5px;
  }

  .clamp {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nowrap {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .empty {
    padding: 30px;
    text-align: center;
  }

  .version {
    width: 130px;
    height: 36px;
  }

  .bar {
    flex: 1;
    height: 8px;
    border-radius: 2px;
    background: var(--bg);
    overflow: hidden;
  }

  .bar div {
    height: 100%;
    background: var(--green);
    transition: width 0.2s;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px;
    border-radius: 4px;
    background: var(--panel-2);
    border: 1px solid var(--border);
  }

  h3 {
    font-size: 14px;
  }

  .client {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .warn {
    font-size: 12px;
    color: var(--warning);
  }
</style>
