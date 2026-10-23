<script lang="ts">
  import { api, on, type Profile, type Project } from "../api";
  import { compactNumber, t } from "../i18n.svelte";
  import { toIconDataUrl } from "../images";
  import { pixelIconKeys } from "../pixelIcons";
  import { addProfile, app, attempt, loadVersions } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import IconPicker from "./IconPicker.svelte";
  import ModIcon from "./ModIcon.svelte";
  import Modal from "./Modal.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  // Left: a live preview of the profile being made. Right: the name and icon,
  // then one of four equal kinds of profile, each with its own small body.
  type Kind = "vanilla" | "fabric" | "modrinth" | "file";
  const kinds: { id: Kind; label: () => string }[] = [
    { id: "vanilla", label: () => "Vanilla" },
    { id: "fabric", label: () => "Fabric" },
    { id: "modrinth", label: () => "Modrinth" },
    { id: "file", label: () => t("create.file") },
  ];

  let kind = $state<Kind>("fabric");
  let busy = $state(false);
  let progress = $state(0);

  // `icon` stays null until the player picks one; until then the preview
  // follows the kind, so a Modrinth pack shows its own picture.
  const defaultIcon = pixelIconKeys[app.profiles.length % pixelIconKeys.length];
  let icon = $state<string | null>(null);
  let picking = $state(false);
  let name = $state("");
  let version = $state("");
  let jars = $state<File[]>([]);
  let packFile = $state<File | null>(null);
  let fileInput = $state<HTMLInputElement>();
  let dragging = $state(false);

  let query = $state("");
  let packs = $state<Project[]>([]);
  let searching = $state(false);
  let pack = $state<Project | null>(null);
  let packVersions = $state<string[]>([]);
  let packVersion = $state("");
  let packIcon = $state<string | null>(null);

  const stem = (file: File) => file.name.replace(/\.(jar|mrpack)$/i, "");
  const choices = $derived(app.versions.filter((v) => kind !== "fabric" || v.fabric));
  const shownIcon = $derived(icon ?? (kind === "modrinth" && pack ? (packIcon ?? "chest") : defaultIcon));
  const fallbackName = $derived.by(() => {
    if (kind === "modrinth") return pack?.title ?? "Modrinth";
    if (kind === "file") return packFile ? stem(packFile) : t("create.modpack");
    if (kind === "fabric" && jars.length) return stem(jars[0]);
    return `${kind === "fabric" ? "Fabric" : "Vanilla"} ${version}`;
  });
  const previewMeta = $derived.by(() => {
    if (kind === "modrinth") return pack ? `${packVersion || "…"} · Modrinth` : "Modrinth";
    if (kind === "file") return "Modrinth · .mrpack";
    return `${version || "…"} · ${kind === "fabric" ? "Fabric" : "Vanilla"}`;
  });
  const ready = $derived(kind === "modrinth" ? !!pack && !!packVersion : kind === "file" ? !!packFile : !!version);

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
    if (kind !== "modrinth") return;
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
    picking = false;
    progress = 0;
    const profile = await attempt(action);
    busy = false;
    if (profile) {
      app.createOpen = false;
      await addProfile(profile);
    }
  }

  // Files decide their own kind: a .mrpack is a modpack, jars ride on Fabric.
  function pickFiles(files: FileList | null | undefined) {
    const list = [...(files ?? [])];
    const mrpack = list.find((f) => f.name.toLowerCase().endsWith(".mrpack"));
    const added = list.filter((f) => f.name.toLowerCase().endsWith(".jar"));
    if (mrpack) {
      packFile = mrpack;
      kind = "file";
    } else if (added.length) {
      jars = [...jars, ...added];
      kind = "fabric";
    }
    if (fileInput) fileInput.value = "";
  }

  function drop(e: DragEvent) {
    e.preventDefault();
    dragging = false;
    pickFiles(e.dataTransfer?.files);
  }

  function create() {
    const chosenName = name.trim();
    const chosenIcon = shownIcon;
    if (kind === "modrinth" && pack) {
      const { slug, title } = pack;
      run(() => api.createModpackProfile(slug, packVersion, chosenName || title, chosenIcon));
    } else if (kind === "file" && packFile) {
      const file = packFile;
      run(() => api.importModpack(file, chosenName, chosenIcon));
    } else {
      const loader = kind === "fabric" ? "fabric" : "vanilla";
      const files = loader === "fabric" ? jars : [];
      run(async () => {
        const profile = await api.createProfile({ name: chosenName || fallbackName, icon: chosenIcon, gameVersion: version, loader, memoryMb: null });
        for (const jar of files) await api.addModFile(profile.id, jar);
        return profile;
      });
    }
  }

  function chooseIcon(value: string) {
    icon = value;
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
        <ProfileIcon icon={shownIcon} size={96} />
        <span class="mc title">{name.trim() || fallbackName}</span>
        <span class="meta">{previewMeta}</span>
        <span class="btn green fake">{t("profile.play")}</span>
      </div>
    </aside>

    <section class="form">
      <div class="head">
        <h2>{t("create.title")}</h2>
        <button class="close" aria-label={t("window.close")} onclick={close}><Icon name="x" size={18} /></button>
      </div>

      <div class="identity">
        <button class="icon-btn" aria-label={t("form.icon")} aria-expanded={picking} onclick={() => (picking = !picking)} disabled={busy}>
          <ProfileIcon icon={shownIcon} size={40} />
        </button>
        <!-- svelte-ignore a11y_autofocus -->
        <input class="input name" bind:value={name} placeholder={fallbackName} maxlength="40" aria-label={t("form.name")} autofocus disabled={busy} />
        {#if picking}
          <div class="picker">
            <IconPicker value={shownIcon} extra={kind === "modrinth" ? packIcon : null} onchange={chooseIcon} />
          </div>
        {/if}
      </div>

      <div class="segmented kinds" role="tablist">
        {#each kinds as k (k.id)}
          <button role="tab" aria-selected={kind === k.id} class:on={kind === k.id} onclick={() => (kind = k.id)} disabled={busy}>{k.label()}</button>
        {/each}
      </div>

      <div class="body">
        {#if kind === "vanilla"}
          <p class="hint center">{t("create.vanillaHint")}</p>
        {:else if kind === "fabric" || kind === "file"}
          <button
            class="drop"
            class:over={dragging}
            onclick={() => fileInput?.click()}
            ondragover={(e) => {
              e.preventDefault();
              dragging = true;
            }}
            ondragleave={() => (dragging = false)}
            ondrop={drop}
            disabled={busy}
          >
            <Icon name="upload" size={20} />
            <span>{kind === "fabric" ? t("create.jars") : t("create.packFile")}</span>
          </button>
          {#if kind === "fabric" && jars.length}
            <div class="files">
              {#each jars as jar, i (jar.name + i)}
                <span class="file">{jar.name}<button aria-label={t("mods.remove")} onclick={() => (jars = jars.filter((_, j) => j !== i))}><Icon name="x" size={13} /></button></span>
              {/each}
            </div>
          {:else if kind === "file" && packFile}
            <div class="files">
              <span class="file">{packFile.name}<button aria-label={t("mods.remove")} onclick={() => (packFile = null)}><Icon name="x" size={13} /></button></span>
            </div>
          {/if}
        {:else}
          <label class="search">
            <Icon name={searching ? "loader" : "search"} size={16} class={searching ? "spin" : ""} />
            <input class="input" bind:value={query} placeholder={t("create.searchModpacks")} spellcheck="false" />
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
        {/if}
      </div>

      <div class="footer">
        {#if kind === "vanilla" || kind === "fabric"}
          <select class="input version" bind:value={version} aria-label={t("form.version")} disabled={busy || !choices.length}>
            {#each choices as choice (choice.id)}<option value={choice.id}>{choice.id}</option>{/each}
          </select>
        {:else if kind === "modrinth"}
          <select class="input version" bind:value={packVersion} aria-label={t("form.version")} disabled={busy || !pack}>
            {#if !packVersions.length}<option value="">{t("form.version")}</option>{/if}
            {#each packVersions as v (v)}<option value={v}>{v}</option>{/each}
          </select>
        {/if}
        <span class="grow">{#if busy}<span class="bar"><span style:width="{progress}%"></span></span>{/if}</span>
        <button class="btn green" onclick={create} disabled={busy || !ready}>
          {#if busy}<Icon name="loader" size={15} class="spin" />{/if}{t("create.button")}
        </button>
      </div>
      <input
        bind:this={fileInput}
        type="file"
        accept={kind === "file" ? ".mrpack" : ".jar"}
        multiple={kind !== "file"}
        hidden
        onchange={(e) => pickFiles(e.currentTarget.files)}
      />
    </section>
  </div>
</Modal>

<style>
  .split {
    display: grid;
    grid-template-columns: 250px minmax(0, 1fr);
    height: 440px;
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
  }

  .head h2 {
    flex: 1;
    font-size: 18px;
  }

  .close {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-2);
  }

  .close:hover {
    background: var(--panel-2);
    color: var(--text);
  }

  .identity {
    position: relative;
    display: flex;
    gap: 8px;
  }

  .icon-btn {
    flex-shrink: 0;
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    border-radius: 6px;
    border: 2px solid transparent;
  }

  .icon-btn:hover:not(:disabled),
  .icon-btn[aria-expanded="true"] {
    border-color: var(--green-text);
  }

  .input.name {
    flex: 1;
    height: 44px;
    font-size: 15px;
  }

  .picker {
    position: absolute;
    z-index: 5;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    padding: 10px;
    border-radius: 6px;
    background: var(--panel);
    border: 1px solid var(--border-strong);
    box-shadow: 0 12px 30px rgb(0 0 0 / 0.5);
  }

  .kinds {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .center {
    margin: auto;
  }

  .drop {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 12px;
    border-radius: 6px;
    border: 2px dashed var(--border-strong);
    color: var(--text-3);
    font-size: 13px;
  }

  .drop:hover:not(:disabled),
  .drop.over {
    border-color: var(--green-text);
    color: var(--text-2);
  }

  .files {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    max-height: 64px;
    overflow-y: auto;
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
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .version {
    width: 150px;
    height: 36px;
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

  .packs {
    flex: 1;
    min-height: 0;
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
</style>
