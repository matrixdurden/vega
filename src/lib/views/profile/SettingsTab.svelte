<script lang="ts">
  import { api, toDraft, type Profile, type ProfileFolder } from "../../api";
  import Icon from "../../components/Icon.svelte";
  import MemorySlider from "../../components/MemorySlider.svelte";
  import { t } from "../../i18n.svelte";
  import { app, attempt, deleteProfile, updateProfile } from "../../state.svelte";

  let { profile }: { profile: Profile } = $props();

  const folders: ProfileFolder[] = ["mods", "saves", "resourcepacks", "shaderpacks", "screenshots"];
  const globalMb = $derived(app.settings?.memoryMb ?? 4096);

  let confirmDelete = $state(false);
  let saveTimer: ReturnType<typeof setTimeout>;

  function setMemory(memoryMb: number | null) {
    const target = app.profiles.find((p) => p.id === profile.id);
    if (!target) return;
    target.memoryMb = memoryMb;
    clearTimeout(saveTimer);
    const id = target.id;
    const draft = toDraft(target);
    saveTimer = setTimeout(() => attempt(() => updateProfile(id, draft)), 400);
  }

  function remove() {
    if (!confirmDelete) {
      confirmDelete = true;
      setTimeout(() => (confirmDelete = false), 4000);
      return;
    }
    deleteProfile(profile.id);
  }
</script>

<div class="card">
  <div class="row">
    <div class="text">
      <span class="label"><Icon name="memory" size={16} />{t("psettings.memory")}</span>
      <span class="hint">{t("psettings.memoryHint")}</span>
    </div>
    <div class="memory">
      <label class="check">
        <input type="checkbox" checked={profile.memoryMb === null} onchange={(e) => setMemory(e.currentTarget.checked ? null : globalMb)} />
        {t("psettings.memoryDefault", { gb: globalMb / 1024 })}
      </label>
      {#if profile.memoryMb !== null}
        <MemorySlider value={profile.memoryMb} onchange={setMemory} />
      {/if}
    </div>
  </div>

  <div class="row">
    <div class="text">
      <span class="label"><Icon name="folder" size={16} />{t("psettings.folders")}</span>
    </div>
    <div class="folders">
      {#each folders as folder (folder)}
        <button class="pill" onclick={() => attempt(() => api.openProfileFolder(profile.id, folder))}>{folder}</button>
      {/each}
    </div>
  </div>

  <div class="row">
    <div class="text">
      <span class="label danger"><Icon name="trash" size={16} />{t("psettings.danger")}</span>
      <span class="hint">{t("psettings.dangerHint")}</span>
    </div>
    <button class="btn danger" class:armed={confirmDelete} onclick={remove}>
      {confirmDelete ? t("psettings.confirmDelete") : t("psettings.delete")}
    </button>
  </div>
</div>

<style>
  .card {
    max-width: 760px;
    border-radius: 16px;
    background: rgb(18 16 42 / 0.6);
    border: 1px solid var(--border);
    backdrop-filter: blur(10px);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 16px 18px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }

  .label :global(svg) {
    color: var(--text-3);
  }

  .label.danger,
  .label.danger :global(svg) {
    color: var(--danger);
  }

  .hint {
    font-size: 12px;
    color: var(--text-3);
  }

  .memory {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 10px;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
    font-size: 13px;
    cursor: pointer;
  }

  .check input {
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
  }

  .folders {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 6px;
  }

  .pill {
    height: 30px;
    padding: 0 12px;
    border-radius: 99px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-2);
    font-size: 12px;
  }

  .pill:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .btn.danger.armed {
    background: var(--danger-soft);
    border-color: rgb(240 149 149 / 0.45);
  }
</style>
