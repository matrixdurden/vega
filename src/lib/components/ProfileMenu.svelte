<script lang="ts">
  import { api } from "../api";
  import { t } from "../i18n.svelte";
  import { app, deleteProfile, errorText, selectProfile, selectedProfile, toast } from "../state.svelte";
  import Icon from "./Icon.svelte";

  let open = $state(false);
  let confirmDelete = $state(false);
  let root: HTMLDivElement;

  const current = $derived(selectedProfile());
  const busy = $derived(app.launch.phase !== "idle");

  function close() {
    open = false;
    confirmDelete = false;
  }

  function onWindowClick(event: MouseEvent) {
    if (open && !root.contains(event.target as Node)) close();
  }

  async function pick(id: string) {
    close();
    await selectProfile(id);
  }

  async function remove() {
    if (!current) return;
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }
    const id = current.id;
    close();
    await deleteProfile(id);
  }

  function openFolder() {
    if (current) api.openProfileFolder(current.id).catch((err) => toast(errorText(err), "error"));
    close();
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={(e) => e.key === "Escape" && close()} />

<div class="menu" bind:this={root}>
  <button class="trigger" disabled={busy} aria-expanded={open} onclick={() => (open = !open)}>
    {#if current}
      <span class="badge" class:fabric={current.loader === "fabric"}>{current.loader === "fabric" ? "Fabric" : "Vanilla"}</span>
      <span class="name">{current.name}</span>
    {:else}
      <span class="name">{t("profile.new")}</span>
    {/if}
    <Icon name="chevron" size={16} />
  </button>

  {#if open}
    <div class="popover" role="menu">
      {#each app.profiles as profile (profile.id)}
        <button role="menuitem" class="item" class:selected={profile.id === app.selectedId} onclick={() => pick(profile.id)}>
          <span class="dot" class:fabric={profile.loader === "fabric"}></span>
          <span class="grow">{profile.name}</span>
          <span class="meta">{profile.gameVersion}</span>
          {#if profile.id === app.selectedId}<Icon name="check" size={16} />{/if}
        </button>
      {/each}
      <div class="divider"></div>
      <button role="menuitem" class="item" onclick={() => ((app.newProfileOpen = true), close())}>
        <Icon name="plus" size={16} />{t("profile.new")}
      </button>
      {#if current}
        <button role="menuitem" class="item" onclick={openFolder}>
          <Icon name="folder" size={16} />{t("profile.openFolder")}
        </button>
        <button role="menuitem" class="item danger" onclick={remove}>
          <Icon name="trash" size={16} />{confirmDelete ? t("profile.confirmDelete") : t("profile.delete")}
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .menu {
    position: relative;
  }

  .trigger {
    height: 52px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 16px 0 12px;
    border-radius: var(--radius);
    background: var(--surface);
    border: 1px solid var(--border);
    backdrop-filter: blur(10px);
    transition: background 0.15s, border-color 0.15s;
  }

  .trigger:hover:not(:disabled) {
    background: var(--surface-hover);
    border-color: var(--border-strong);
  }

  .badge {
    padding: 3px 8px;
    border-radius: 7px;
    font-size: 11px;
    font-weight: 600;
    background: var(--surface-hover);
    color: var(--text-2);
  }

  .badge.fabric {
    background: var(--accent-soft);
    color: #cecbf6;
  }

  .name {
    font-weight: 500;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .popover {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    min-width: 280px;
    max-height: 360px;
    overflow-y: auto;
    padding: 6px;
    border-radius: var(--radius);
    background: var(--bg-raised);
    border: 1px solid var(--border-strong);
    box-shadow: 0 20px 50px rgb(0 0 0 / 0.5);
    animation: pop 0.16s var(--ease);
    z-index: 30;
  }

  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(6px) scale(0.98);
    }
  }

  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: 9px;
    text-align: left;
    color: var(--text-2);
  }

  .item:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .item.selected {
    color: var(--text);
  }

  .item.danger {
    color: var(--danger);
  }

  .item.danger:hover {
    background: var(--danger-soft);
  }

  .grow {
    flex: 1;
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-3);
  }

  .dot.fabric {
    background: var(--accent);
  }

  .divider {
    height: 1px;
    margin: 6px 4px;
    background: var(--border);
  }
</style>
