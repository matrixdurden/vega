<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { api, type ModResult, type Profile } from "../api";
  import { compactNumber, t } from "../i18n.svelte";
  import { app, errorText, toast } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ModIcon from "./ModIcon.svelte";

  let { profile }: { profile: Profile } = $props();

  let query = $state("");
  let results = $state<ModResult[]>([]);
  let searching = $state(false);
  const busy = new SvelteSet<string>();

  // Debounced search; an empty query shows the most popular mods.
  $effect(() => {
    const q = query;
    const id = profile.id;
    searching = true;
    const timer = setTimeout(
      async () => {
        try {
          const hits = await api.searchMods(id, q);
          if (q === query) results = hits;
        } catch (err) {
          toast(errorText(err), "error");
        } finally {
          if (q === query) searching = false;
        }
      },
      q ? 300 : 0,
    );
    return () => clearTimeout(timer);
  });

  async function toggle(mod: ModResult) {
    if (busy.has(mod.projectId)) return;
    busy.add(mod.projectId);
    try {
      app.mods = mod.installed ? await api.removeMod(profile.id, mod.projectId) : await api.installMod(profile.id, mod.projectId);
      for (const result of results) result.installed = app.mods.some((m) => m.projectId === result.projectId);
      if (mod.installed) toast(t("mods.installedToast", { name: mod.title }));
    } catch (err) {
      toast(errorText(err), "error");
    } finally {
      busy.delete(mod.projectId);
    }
  }

  const close = () => (app.modBrowserOpen = false);
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} />

<section class="browser" aria-labelledby="mod-browser-title">
  <header>
    <div>
      <h2 id="mod-browser-title">{t("mods.add")}</h2>
      <p>{t("mods.for", { profile: profile.name, version: profile.gameVersion })}</p>
    </div>
    <button class="close" aria-label={t("window.close")} onclick={close}><Icon name="x" /></button>
  </header>

  <label class="search">
    <Icon name={searching ? "loader" : "search"} size={18} class={searching ? "spin" : ""} />
    <!-- svelte-ignore a11y_autofocus -->
    <input class="input" bind:value={query} placeholder={t("mods.search")} spellcheck="false" autofocus />
  </label>

  <div class="list">
    {#each results as mod (mod.projectId)}
      <article class="row">
        <ModIcon url={mod.iconUrl} title={mod.title} size={46} />
        <div class="info">
          <div class="name">
            <b>{mod.title}</b>
            <span class="meta">{mod.author}</span>
          </div>
          <p class="desc">{mod.description}</p>
        </div>
        <span class="downloads"><Icon name="download" size={14} />{compactNumber(mod.downloads)}</span>
        <button class="btn" class:green={!mod.installed} class:added={mod.installed} disabled={busy.has(mod.projectId)} onclick={() => toggle(mod)}>
          {#if busy.has(mod.projectId)}
            <Icon name="loader" size={16} class="spin" />
          {:else}
            <Icon name={mod.installed ? "check" : "plus"} size={16} />
          {/if}
          {mod.installed ? t("mods.added") : t("mods.install")}
        </button>
      </article>
    {:else}
      {#if !searching}<p class="none">{t("mods.empty")}</p>{/if}
    {/each}
  </div>
</section>

<style>
  .browser {
    position: absolute;
    inset: 0;
    z-index: 12;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 44px 32px 0;
    background: var(--bg);
    animation: slide 0.22s var(--ease);
  }

  @keyframes slide {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }

  h2 {
    font-size: 26px;
  }

  header p {
    color: var(--text-3);
    font-size: 13px;
  }

  .close {
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    background: var(--panel-2);
    color: var(--text-2);
  }

  .close:hover {
    background: var(--panel-2);
    color: var(--text);
  }

  .search {
    position: relative;
    display: block;
    color: var(--text-3);
  }

  .search :global(svg) {
    position: absolute;
    z-index: 1;
    left: 14px;
    top: 13px;
  }

  .search .input {
    height: 44px;
    padding-left: 42px;
    font-size: 15px;
  }

  .list {
    flex: 1;
    overflow-y: auto;
    margin: 0 -8px;
    padding: 0 8px 32px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 12px;
    border-radius: var(--radius);
    background: var(--panel);
    border: 2px solid transparent;
  }

  .row:hover {
    border-color: var(--border-strong);
  }

  .info {
    flex: 1;
    min-width: 0;
  }

  .name {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .name b {
    font-weight: 600;
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
  }

  .desc {
    margin-top: 2px;
    font-size: 13px;
    color: var(--text-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .downloads {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }

  .btn {
    min-width: 96px;
    height: 36px;
  }

  .btn.added {
    color: var(--green-text);
  }

  .btn.added:hover:not(:disabled) {
    color: var(--danger);
  }

  .none {
    padding: 40px 0;
    text-align: center;
    color: var(--text-3);
  }
</style>
