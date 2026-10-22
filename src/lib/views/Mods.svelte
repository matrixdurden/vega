<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { api, type InstalledMod, type ModResult } from "../api";
  import Icon from "../components/Icon.svelte";
  import { compactNumber, t } from "../i18n.svelte";
  import { app, errorText, selectedProfile, toast } from "../state.svelte";

  let query = $state("");
  let tab = $state<"discover" | "installed">("discover");
  let results = $state<ModResult[]>([]);
  let searching = $state(false);
  const busy = new SvelteSet<string>();

  const profile = $derived(selectedProfile());
  const modded = $derived(profile?.loader === "fabric");

  // Debounced search; an empty query shows the most popular mods.
  $effect(() => {
    const q = query;
    const id = profile?.id;
    if (!id || !modded) return;
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

  function syncInstalled(list: InstalledMod[]) {
    app.mods = list;
    for (const result of results) result.installed = list.some((mod) => mod.projectId === result.projectId);
  }

  async function install(mod: ModResult) {
    if (!profile || busy.has(mod.projectId)) return;
    busy.add(mod.projectId);
    try {
      syncInstalled(await api.installMod(profile.id, mod.projectId));
      toast(t("mods.installedToast", { name: mod.title }));
    } catch (err) {
      toast(errorText(err), "error");
    } finally {
      busy.delete(mod.projectId);
    }
  }

  async function remove(projectId: string) {
    if (!profile || busy.has(projectId)) return;
    busy.add(projectId);
    try {
      syncInstalled(await api.removeMod(profile.id, projectId));
    } catch (err) {
      toast(errorText(err), "error");
    } finally {
      busy.delete(projectId);
    }
  }
</script>

{#snippet modIcon(url: string | null, title: string)}
  {#if url}
    <img class="icon" src={url} alt="" loading="lazy" />
  {:else}
    <div class="icon placeholder">{title.slice(0, 1)}</div>
  {/if}
{/snippet}

<section class="mods">
  <header>
    <div>
      <h1>{t("mods.title")}</h1>
      {#if profile}<p class="sub">{profile.name} · Minecraft {profile.gameVersion}</p>{/if}
    </div>
    {#if modded}
      <div class="tabs" role="tablist">
        <button role="tab" aria-selected={tab === "discover"} class:on={tab === "discover"} onclick={() => (tab = "discover")}>
          {t("mods.discover")}
        </button>
        <button role="tab" aria-selected={tab === "installed"} class:on={tab === "installed"} onclick={() => (tab = "installed")}>
          {t("mods.installed")} <span class="count">{app.mods.length}</span>
        </button>
      </div>
    {/if}
  </header>

  {#if !modded}
    <div class="empty">
      <Icon name="puzzle" size={32} />
      <h2>{t("mods.vanillaTitle")}</h2>
      <p>{t("mods.vanillaBody")}</p>
      <button class="btn primary" onclick={() => (app.newProfileOpen = true)}><Icon name="plus" size={16} />{t("profile.new")}</button>
    </div>
  {:else if tab === "discover"}
    <label class="search">
      <Icon name={searching ? "loader" : "search"} size={18} class={searching ? "spin" : ""} />
      <input class="input" bind:value={query} placeholder={t("mods.search")} spellcheck="false" />
    </label>

    <div class="list">
      {#each results as mod (mod.projectId)}
        <article class="row">
          {@render modIcon(mod.iconUrl, mod.title)}
          <div class="info">
            <div class="title">
              <b>{mod.title}</b>
              <span class="meta">{t("mods.by", { author: mod.author })}</span>
            </div>
            <p class="desc">{mod.description}</p>
          </div>
          <span class="downloads"><Icon name="download" size={14} />{compactNumber(mod.downloads)}</span>
          {#if mod.installed}
            <button class="btn added" onclick={() => remove(mod.projectId)} disabled={busy.has(mod.projectId)}>
              <Icon name="check" size={16} />{t("mods.added")}
            </button>
          {:else}
            <button class="btn primary" onclick={() => install(mod)} disabled={busy.has(mod.projectId)}>
              {#if busy.has(mod.projectId)}<Icon name="loader" size={16} class="spin" />{:else}<Icon name="plus" size={16} />{/if}
              {t("mods.install")}
            </button>
          {/if}
        </article>
      {:else}
        {#if !searching}<p class="none">{t("mods.empty")}</p>{/if}
      {/each}
    </div>
  {:else}
    <div class="list">
      {#each app.mods as mod (mod.projectId)}
        <article class="row">
          {@render modIcon(mod.iconUrl, mod.title)}
          <div class="info">
            <div class="title">
              <b>{mod.title}</b>
              {#if mod.dependency}<span class="tag">{t("mods.dependency")}</span>{/if}
            </div>
            <p class="desc">{mod.versionNumber} · {mod.filename}</p>
          </div>
          <button class="btn danger" onclick={() => remove(mod.projectId)} disabled={busy.has(mod.projectId)}>
            <Icon name="trash" size={16} />{t("mods.remove")}
          </button>
        </article>
      {:else}
        <p class="none">{t("mods.noneInstalled")}</p>
      {/each}
    </div>
  {/if}
</section>

<style>
  .mods {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 36px 44px 0;
  }

  header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
  }

  h1 {
    font-size: 28px;
    font-weight: 650;
    letter-spacing: -0.02em;
  }

  .sub {
    color: var(--text-3);
    font-size: 13px;
  }

  .tabs {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .tabs button {
    height: 32px;
    padding: 0 14px;
    border-radius: 8px;
    color: var(--text-2);
    font-weight: 500;
  }

  .tabs button.on {
    background: var(--accent);
    color: #fff;
  }

  .count {
    margin-left: 4px;
    font-size: 12px;
    opacity: 0.8;
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
    backdrop-filter: blur(10px);
  }

  .list {
    flex: 1;
    overflow-y: auto;
    margin: 0 -8px;
    padding: 0 8px 32px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
    border-radius: var(--radius);
    background: rgb(18 16 42 / 0.6);
    border: 1px solid var(--border);
    backdrop-filter: blur(10px);
    transition: border-color 0.15s;
  }

  .row:hover {
    border-color: var(--border-strong);
  }

  .icon {
    width: 46px;
    height: 46px;
    flex-shrink: 0;
    border-radius: 11px;
    object-fit: cover;
    background: var(--surface);
  }

  .placeholder {
    display: grid;
    place-items: center;
    font-size: 18px;
    font-weight: 600;
    color: #cecbf6;
    background: var(--accent-soft);
  }

  .info {
    flex: 1;
    min-width: 0;
  }

  .title {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .title b {
    font-weight: 600;
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
  }

  .tag {
    padding: 1px 7px;
    border-radius: 6px;
    background: var(--surface-hover);
    color: var(--text-3);
    font-size: 11px;
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
    color: var(--success);
  }

  .btn.added:hover:not(:disabled) {
    color: var(--danger);
  }

  .none {
    padding: 40px 0;
    text-align: center;
    color: var(--text-3);
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding-bottom: 60px;
    color: var(--text-3);
    text-align: center;
  }

  .empty h2 {
    color: var(--text);
    font-size: 18px;
    font-weight: 600;
  }

  .empty p {
    margin-bottom: 8px;
  }
</style>
