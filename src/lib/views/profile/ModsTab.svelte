<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { api, type Profile } from "../../api";
  import Icon from "../../components/Icon.svelte";
  import ModIcon from "../../components/ModIcon.svelte";
  import { t } from "../../i18n.svelte";
  import { app, attempt } from "../../state.svelte";

  let { profile }: { profile: Profile } = $props();

  const busy = new SvelteSet<string>();
  // Mods the player picked first, then the libraries they pulled in.
  const mods = $derived([...app.mods].sort((a, b) => Number(a.dependency) - Number(b.dependency)));

  async function run(projectId: string, action: () => ReturnType<typeof api.removeMod>) {
    if (busy.has(projectId)) return;
    busy.add(projectId);
    const list = await attempt(action);
    if (list) app.mods = list;
    busy.delete(projectId);
  }
</script>

{#if profile.loader === "vanilla"}
  <p class="empty">{t("mods.vanilla")}</p>
{:else if mods.length === 0}
  <div class="empty">
    <p>{t("mods.none")}</p>
    <button class="btn green" onclick={() => (app.modBrowserOpen = true)}><Icon name="plus" size={16} />{t("mods.add")}</button>
  </div>
{:else}
  <div class="list">
    {#each mods as mod (mod.projectId)}
      <article class="row" class:off={!mod.enabled}>
        <ModIcon url={mod.iconUrl} title={mod.title} size={34} />
        <div class="info">
          <b>{mod.title}</b>
          {#if mod.dependency}<span class="tag">{t("mods.dependency")}</span>{/if}
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

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 10px;
    border-radius: 6px;
    background: var(--panel);
    border: 2px solid transparent;
  }

  .row:hover {
    border-color: #2b2850;
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
    gap: 8px;
  }

  .info b {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag {
    font-size: 11px;
    color: var(--text-3);
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
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 48px 20px;
    color: var(--text-3);
    text-align: center;
  }
</style>
