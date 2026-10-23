<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { api, type InstalledMod, type Profile } from "../../api";
  import Icon from "../../components/Icon.svelte";
  import ModIcon from "../../components/ModIcon.svelte";
  import { t } from "../../i18n.svelte";
  import { app, attempt, toast } from "../../state.svelte";

  let { profile }: { profile: Profile } = $props();

  const busy = new SvelteSet<string>();
  let dragging = $state(false);
  let uploading = $state(false);
  let input = $state<HTMLInputElement>();

  // Mods the player picked first, then the libraries they pulled in.
  const mods = $derived([...app.mods].sort((a, b) => Number(a.dependency) - Number(b.dependency)));

  async function run(projectId: string, action: () => Promise<InstalledMod[]>) {
    if (busy.has(projectId)) return;
    busy.add(projectId);
    const list = await attempt(action);
    if (list) app.mods = list;
    busy.delete(projectId);
  }

  async function addFiles(files: Iterable<File>) {
    const jars = [...files].filter((file) => file.name.toLowerCase().endsWith(".jar"));
    if (!jars.length || uploading) return;
    uploading = true;
    let added = 0;
    for (const jar of jars) {
      const list = await attempt(() => api.addModFile(profile.id, jar));
      if (list) {
        app.mods = list;
        added++;
      }
    }
    uploading = false;
    if (added) toast(t("mods.filesToast", { n: added }));
  }

  function onDrop(event: DragEvent) {
    event.preventDefault();
    dragging = false;
    if (event.dataTransfer) addFiles(event.dataTransfer.files);
  }
</script>

{#if profile.loader === "vanilla"}
  <p class="empty">{t("mods.vanilla")}</p>
{:else}
  <div
    class="zone"
    class:dragging
    role="region"
    aria-label={t("tab.mods")}
    ondragover={(e) => (e.preventDefault(), (dragging = true))}
    ondragleave={(e) => e.currentTarget === e.target && (dragging = false)}
    ondrop={onDrop}
  >
    <div class="toolbar">
      <button class="btn" onclick={() => (app.modBrowserOpen = true)}><Icon name="search" size={15} />{t("mods.add")}</button>
      <button class="btn" onclick={() => input?.click()} disabled={uploading}>
        <Icon name={uploading ? "loader" : "upload"} size={15} class={uploading ? "spin" : ""} />{t("mods.addFile")}
      </button>
      <input bind:this={input} type="file" accept=".jar" multiple hidden onchange={(e) => (addFiles(e.currentTarget.files ?? []), (e.currentTarget.value = ""))} />
    </div>

    {#if mods.length === 0}
      <p class="empty">{t("mods.none")}</p>
    {:else}
      <div class="list">
        {#each mods as mod (mod.projectId)}
          <article class="row" class:off={!mod.enabled}>
            <ModIcon url={mod.iconUrl} title={mod.title} size={34} />
            <div class="info">
              <b>{mod.title}</b>
              {#if mod.versionNumber}<span class="meta">{mod.versionNumber}</span>{/if}
              {#if mod.dependency}<span class="tag">{t("mods.dependency")}</span>{/if}
              {#if mod.projectId.startsWith("file:")}<span class="tag">{t("mods.local")}</span>{/if}
            </div>
            <button
              class="switch"
              role="switch"
              aria-checked={mod.enabled}
              aria-label={t("mods.enable")}
              title={t("mods.enable")}
              disabled={busy.has(mod.projectId)}
              onclick={() => run(mod.projectId, () => api.setModEnabled(profile.id, mod.projectId, !mod.enabled))}
            ></button>
            <button
              class="remove"
              aria-label={t("mods.remove")}
              title={t("mods.remove")}
              disabled={busy.has(mod.projectId)}
              onclick={() => run(mod.projectId, () => api.removeMod(profile.id, mod.projectId))}
            >
              <Icon name="trash" size={16} />
            </button>
          </article>
        {/each}
      </div>
    {/if}

    {#if dragging}
      <div class="drop"><Icon name="upload" size={28} /><span class="mc">{t("mods.drop")}</span></div>
    {/if}
  </div>
{/if}

<style>
  .zone {
    position: relative;
    min-height: 100%;
  }

  .toolbar {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: 4px;
    background: var(--panel);
    border: 1px solid var(--border);
  }

  .row:hover {
    border-color: var(--border-strong);
  }

  .row.off .info,
  .row.off :global(.mod-icon) {
    opacity: 0.4;
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .info b {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
    white-space: nowrap;
  }

  .tag {
    padding: 0 6px;
    border-radius: 3px;
    background: var(--panel-2);
    color: var(--text-2);
    font-size: 11px;
  }

  .remove {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-3);
  }

  .remove:hover:not(:disabled) {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .empty {
    padding: 40px 20px;
    text-align: center;
    color: var(--text-3);
  }

  .drop {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    border: 2px dashed var(--green-text);
    border-radius: 6px;
    background: rgb(23 23 27 / 0.9);
    color: var(--green-text);
    pointer-events: none;
  }
</style>
