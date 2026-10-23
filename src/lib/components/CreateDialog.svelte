<script lang="ts">
  import { api, on, type Loader, type Profile, type Project } from "../api";
  import { compactNumber, t } from "../i18n.svelte";
  import { toIconDataUrl } from "../images";
  import { pixelIconKeys } from "../pixelIcons";
  import { addProfile, app, attempt, loadVersions } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import IconPicker from "./IconPicker.svelte";
  import ModIcon from "./ModIcon.svelte";
  import Modal from "./Modal.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  // Left: a live preview of the profile being made. Right: the fields, or a
  // Modrinth modpack search that swaps in for them.
  let view = $state<"form" | "modrinth">("form");
  let busy = $state(false);
  let progress = $state(0);

  let icon = $state<string>(pixelIconKeys[app.profiles.length % pixelIconKeys.length]);
  let name = $state("");
  let loader = $state<Loader>("fabric");
  let version = $state("");
  let jars = $state<File[]>([]);
  let fileInput = $state<HTMLInputElement>();

  let query = $state("");
  let packs = $state<Project[]>([]);
  let searching = $state(false);
  let pack = $state<Project | null>(null);
  let packVersions = $state<string[]>([]);
  let packVersion = $state("");
  // The pack's own icon as a data URL, offered next to the pixel icons.
  let packIcon = $state<string | null>(null);
  // Once the player picks an icon, a pack's icon no longer replaces it.
  let iconChosen = false;
  let picking = $state(false);

  const choices = $derived(app.versions.filter((v) => loader === "vanilla" || v.fabric));
  const fallbackName = $derived(
    view === "modrinth" && pack
      ? pack.title
      : (jars[0]?.name.replace(/\.jar$/i, "") ?? `${loader === "fabric" ? "Fabric" : "Vanilla"} ${version}`),
  );
  const previewName = $derived(name.trim() || fallbackName);
  const previewMeta = $derived(
    view === "modrinth" ? (pack ? `${packVersion || "…"} · Fabric` : "") : `${version || "…"} · ${loader === "fabric" ? "Fabric" : "Vanilla"}`,
  );

  // Fixed stars for the preview sky.
  const stars = Array.from({ length: 26 }, (_, i) => ({ x: (i * 37) % 100, y: (i * 53) % 70, big: i % 5 === 0 }));

  $effect(() => {
    loadVersions();
  });

  $effect(() => {
    if (choices.length && !choices.some((v) => v.id === version)) version = choices[0].id;
  });

  $effect(() => {
    const off = on("install-progress", (e) => (progress = e.total ? Math.round((e.done / e.total) * 100) : 0));
    return () => void off.then((unlisten) => unlisten());
  });

  $effect(() => {
    if (view !== "modrinth") return;
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

  // A .mrpack becomes its own profile; jars ride along with the form.
  function pickFiles(files: FileList | null) {
    const list = [...(files ?? [])];
    const mrpack = list.find((f) => f.name.toLowerCase().endsWith(".mrpack"));
    if (mrpack) run(() => api.importModpack(mrpack, "chest"));
    else if (list.length) {
      jars = [...jars, ...list.filter((f) => f.name.toLowerCase().endsWith(".jar"))];
      loader = "fabric";
    }
    if (fileInput) fileInput.value = "";
  }

  const create = () =>
    run(async () => {
      const profile = await api.createProfile({ name: name.trim() || fallbackName, icon, gameVersion: version, loader, memoryMb: null });
      for (const jar of jars) await api.addModFile(profile.id, jar);
      return profile;
    });

  const install = () => pack && run(() => api.createModpackProfile(pack!.slug, packVersion, name.trim() || pack!.title, icon));

  function chooseIcon(value: string) {
    icon = value;
    iconChosen = true;
    picking = false;
  }

  async function pickPack(p: Project) {
    pack = p;
    packVersions = [];
    packVersion = "";
    packIcon = null;
    const [versions, picture] = await Promise.all([
      attempt(() => api.modpackVersions(p.slug)),
      p.iconUrl ? toIconDataUrl(p.iconUrl).catch(() => null) : Promise.resolve(null),
    ]);
    if (pack !== p) return;
    packVersions = versions ?? [];
    packVersion = packVersions[0] ?? "";
    packIcon = picture;
    if (!iconChosen) icon = picture ?? "chest";
  }

  const close = () => !busy && (app.createOpen = false);
</script>

<Modal title={t("create.title")} width={780} bare onclose={close}>
  <div class="split">
    <aside class="preview" aria-hidden="true">
      {#each stars as star, i (i)}
        <span class="star" class:big={star.big} style:left="{star.x}%" style:top="{star.y}%"></span>
      {/each}
      <div class="card">
        <ProfileIcon {icon} size={96} />
        <span class="mc title">{previewName}</span>
        <span class="meta">{previewMeta}</span>
        <span class="btn green fake">{t("profile.play")}</span>
      </div>
    </aside>

    <section class="form">
      <div class="head">
        {#if view === "modrinth"}
          <button class="back" aria-label={t("create.back")} onclick={() => (view = "form")} disabled={busy}><Icon name="arrow" size={18} /></button>
        {/if}
        <h2>{view === "modrinth" ? t("create.modrinth") : t("create.title")}</h2>
        <button class="back" aria-label={t("window.close")} onclick={close}><Icon name="x" size={18} /></button>
      </div>

      {#if view === "form"}
        <!-- svelte-ignore a11y_autofocus -->
        <input class="input name" bind:value={name} placeholder={fallbackName} maxlength="40" aria-label={t("form.name")} autofocus />

        <div class="line">
          <div class="segmented">
            <button class:on={loader === "fabric"} onclick={() => (loader = "fabric")}>Fabric</button>
            <button class:on={loader === "vanilla"} onclick={() => (loader = "vanilla")} disabled={jars.length > 0}>Vanilla</button>
          </div>
          <select class="input" bind:value={version} aria-label={t("form.version")} disabled={!choices.length}>
            {#each choices as choice (choice.id)}<option value={choice.id}>{choice.id}</option>{/each}
          </select>
        </div>

        <IconPicker value={icon} onchange={chooseIcon} />

        {#if jars.length}
          <div class="files">
            {#each jars as jar, i (jar.name + i)}
              <span class="file">{jar.name}<button aria-label={t("mods.remove")} onclick={() => (jars = jars.filter((_, j) => j !== i))}><Icon name="x" size={13} /></button></span>
            {/each}
          </div>
        {/if}

        <div class="footer">
          <button class="btn" onclick={() => fileInput?.click()} disabled={busy}><Icon name="upload" size={15} />{t("create.fromFile")}</button>
          <button class="btn" onclick={() => (view = "modrinth")} disabled={busy}><Icon name="search" size={15} />Modrinth</button>
          <span class="grow">{#if busy}<span class="bar"><span style:width="{progress}%"></span></span>{/if}</span>
          <button class="btn green" onclick={create} disabled={busy || !version}>
            {#if busy}<Icon name="loader" size={15} class="spin" />{/if}{t("create.button")}
          </button>
        </div>
        <input bind:this={fileInput} type="file" accept=".jar,.mrpack" multiple hidden onchange={(e) => pickFiles(e.currentTarget.files)} />
      {:else}
        <label class="search">
          <Icon name={searching ? "loader" : "search"} size={16} class={searching ? "spin" : ""} />
          <!-- svelte-ignore a11y_autofocus -->
          <input class="input" bind:value={query} placeholder={t("create.searchModpacks")} spellcheck="false" autofocus />
        </label>
        <div class="packs">
          {#each packs as p (p.projectId)}
            <button class="pack" class:on={pack?.projectId === p.projectId} onclick={() => pickPack(p)} disabled={busy}>
              <ModIcon url={p.iconUrl} title={p.title} size={34} />
              <span class="text"><b class="mc">{p.title}</b><span class="hint">{p.description}</span></span>
              <span class="hint downloads"><Icon name="download" size={13} />{compactNumber(p.downloads)}</span>
            </button>
          {/each}
        </div>
        {#if pack}
          <!-- The pack only suggests a name and icon; both stay editable. -->
          <div class="line">
            <button class="icon-btn" aria-label={t("form.icon")} aria-expanded={picking} onclick={() => (picking = !picking)}>
              <ProfileIcon {icon} size={38} />
            </button>
            <input class="input" bind:value={name} placeholder={pack.title} maxlength="40" aria-label={t("form.name")} />
          </div>
          {#if picking}<IconPicker value={icon} extra={packIcon} onchange={chooseIcon} />{/if}
        {/if}
        <div class="footer">
          <select class="input pack-version" bind:value={packVersion} disabled={busy || !pack} aria-label={t("form.version")}>
            {#if !packVersions.length}<option value="">{t("form.version")}</option>{/if}
            {#each packVersions as v (v)}<option value={v}>{v}</option>{/each}
          </select>
          <span class="grow">{#if busy}<span class="bar"><span style:width="{progress}%"></span></span>{/if}</span>
          <button class="btn green" disabled={busy || !pack || !packVersion} onclick={install}>
            {#if busy}<Icon name="loader" size={15} class="spin" />{/if}{t("create.install")}
          </button>
        </div>
      {/if}
    </section>
  </div>
</Modal>

<style>
  .split {
    display: grid;
    grid-template-columns: 250px minmax(0, 1fr);
    min-height: 400px;
  }

  .preview {
    position: relative;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: linear-gradient(#101026, #262150 70%, #2e2864);
    border-right: 1px solid var(--border);
  }

  .star {
    position: absolute;
    width: 2px;
    height: 2px;
    background: #fff;
    opacity: 0.55;
  }

  .star.big {
    width: 4px;
    height: 4px;
    opacity: 0.8;
  }

  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 0 16px;
    text-align: center;
  }

  .title {
    max-width: 210px;
    margin-top: 6px;
    font-size: 18px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-shadow: 0 2px 0 rgb(0 0 0 / 0.45);
  }

  .meta {
    min-height: 1.5em;
    color: var(--text-2);
    font-size: 12.5px;
  }

  .fake {
    width: 140px;
    height: 40px;
    margin-top: 8px;
    font-size: 15px;
    pointer-events: none;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 20px 20px;
    min-width: 0;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .head h2 {
    flex: 1;
    font-size: 18px;
  }

  .back {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-2);
  }

  .back:hover {
    background: var(--panel-2);
    color: var(--text);
  }

  .input.name {
    height: 44px;
    font-size: 15px;
  }

  .line,
  .footer {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .line .segmented {
    width: 190px;
    flex-shrink: 0;
  }

  .line select {
    flex: 1;
  }

  .files {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .file {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 4px 0 10px;
    border-radius: 4px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    font-size: 12.5px;
  }

  .file button {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    color: var(--text-3);
  }

  .file button:hover {
    color: var(--danger);
  }

  .footer {
    margin-top: auto;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .bar {
    display: block;
    height: 8px;
    border-radius: 2px;
    background: var(--bg);
    overflow: hidden;
  }

  .bar span {
    display: block;
    height: 100%;
    background: var(--green);
    transition: width 0.2s;
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
    padding-left: 36px;
  }

  .icon-btn {
    flex-shrink: 0;
    border-radius: 6px;
    border: 2px solid transparent;
  }

  .icon-btn:hover,
  .icon-btn[aria-expanded="true"] {
    border-color: var(--green-text);
  }

  .packs {
    height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .pack {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
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
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .text b {
    font-size: 13px;
  }

  .text .hint {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .downloads {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .pack-version {
    width: 140px;
    height: 36px;
  }
</style>
