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
  <div class="empty">
    <Icon name="puzzle" size={30} />
    <h2>{t("mods.vanillaTitle")}</h2>
    <p>{t("mods.vanillaBody")}</p>
    <button class="btn" onclick={() => (app.dialog = { mode: "edit", id: profile.id })}><Icon name="pencil" size={16} />{t("profile.edit")}</button>
  </div>
{:else if mods.length === 0}
  <div class="empty">
    <Icon name="puzzle" size={30} />
    <h2>{t("mods.none")}</h2>
    <p>{t("mods.noneBody")}</p>
    <button class="btn primary" onclick={() => (app.modBrowserOpen = true)}><Icon name="plus" size={16} />{t("mods.add")}</button>
  </div>
{:else}
  <div class="list">
    {#each mods as mod (mod.projectId)}
      <article class="row" class:off={!mod.enabled}>
        <ModIcon url={mod.iconUrl} title={mod.title} size={38} />
        <div class="info">
          <div class="name">
            <b>{mod.title}</b>
            {#if mod.dependency}<span class="tag">{t("mods.dependency")}</span>{/if}
          </div>
          <span class="meta">{mod.versionNumber}</span>
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
          <Icon name="trash" size={17} />
        </button>
      </article>
    {/each}
  </div>
{/if}

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 12px;
    border-radius: 12px;
    background: rgb(18 16 42 / 0.55);
    border: 1px solid var(--border);
    backdrop-filter: blur(8px);
    transition: opacity 0.15s, border-color 0.15s;
  }

  .row:hover {
    border-color: var(--border-strong);
  }

  .row.off .info,
  .row.off :global(.mod-icon) {
    opacity: 0.45;
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

  .tag {
    padding: 1px 7px;
    border-radius: 6px;
    background: var(--surface-hover);
    color: var(--text-3);
    font-size: 11px;
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
  }

  .switch {
    position: relative;
    width: 38px;
    height: 22px;
    flex-shrink: 0;
    border-radius: 99px;
    background: rgb(255 255 255 / 0.14);
    transition: background 0.2s;
  }

  .switch::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s var(--ease);
  }

  .switch[aria-checked="true"] {
    background: var(--accent);
  }

  .switch[aria-checked="true"]::after {
    transform: translateX(16px);
  }

  .remove {
    width: 34px;
    height: 34px;
    display: grid;
    place-items: center;
    border-radius: 9px;
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
    gap: 8px;
    padding: 56px 20px;
    text-align: center;
    color: var(--text-3);
  }

  .empty h2 {
    color: var(--text);
    font-size: 17px;
    font-weight: 600;
  }

  .empty p {
    max-width: 360px;
    margin-bottom: 8px;
  }
</style>
