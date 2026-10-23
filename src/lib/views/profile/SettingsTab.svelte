<script lang="ts">
  import { untrack } from "svelte";
  import { api, toDraft, type Loader, type Profile, type ProfileFolder } from "../../api";
  import Icon from "../../components/Icon.svelte";
  import MemorySlider from "../../components/MemorySlider.svelte";
  import ProfileFields from "../../components/ProfileFields.svelte";
  import { t } from "../../i18n.svelte";
  import { app, attempt, deleteProfile, toast, updateProfile } from "../../state.svelte";

  let { profile }: { profile: Profile } = $props();

  const FOLDERS: ProfileFolder[] = ["mods", "resourcepacks", "shaderpacks", "screenshots", "saves"];

  // The form edits a copy; Profile view re-creates this tab per profile.
  const initial = untrack(() => toDraft(profile));
  let icon = $state(initial.icon);
  let name = $state(initial.name);
  let loader = $state<Loader>(initial.loader);
  let version = $state(initial.gameVersion);
  let saving = $state(false);
  let confirmDelete = $state(false);

  const globalMb = $derived(app.settings?.memoryMb ?? 4096);
  const dirty = $derived(icon !== profile.icon || name.trim() !== profile.name || loader !== profile.loader || version !== profile.gameVersion);
  const versionChanged = $derived(version !== profile.gameVersion || loader !== profile.loader);

  async function save() {
    saving = true;
    const ok = await attempt(() => updateProfile(profile.id, { ...toDraft(profile), icon, name, loader, gameVersion: version }));
    saving = false;
    if (ok !== undefined) toast(t("form.saved"));
  }

  let memoryTimer: ReturnType<typeof setTimeout>;
  function setMemory(memoryMb: number | null) {
    const target = app.profiles.find((p) => p.id === profile.id);
    if (!target) return;
    target.memoryMb = memoryMb;
    clearTimeout(memoryTimer);
    const draft = toDraft(target);
    memoryTimer = setTimeout(() => attempt(() => updateProfile(target.id, draft)), 400);
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

<div class="stack">
  <section class="card">
    <h3>{t("psettings.profile")}</h3>
    <ProfileFields bind:icon bind:name bind:loader bind:version />
    <div class="save">
      {#if versionChanged && app.mods.length}<span class="warn"><Icon name="alert" size={14} />{t("form.modsWarning")}</span>{/if}
      <button class="btn green" disabled={!dirty || saving} onclick={save}>
        {#if saving}<Icon name="loader" size={15} class="spin" />{/if}{t("form.save")}
      </button>
    </div>
  </section>

  <section class="card">
    <h3>{t("psettings.memory")}</h3>
    <div class="line">
      <label class="check">
        <input type="checkbox" checked={profile.memoryMb === null} onchange={(e) => setMemory(e.currentTarget.checked ? null : globalMb)} />
        {t("psettings.memoryDefault", { gb: globalMb / 1024 })}
      </label>
      {#if profile.memoryMb !== null}<MemorySlider value={profile.memoryMb} onchange={setMemory} />{/if}
    </div>
  </section>

  <section class="card">
    <h3>{t("psettings.folders")}</h3>
    <div class="folders">
      <button class="btn" onclick={() => attempt(() => api.openProfileFolder(profile.id))}><Icon name="folder" size={15} />{t("psettings.openFolder")}</button>
      {#each FOLDERS as folder (folder)}
        <button class="btn" onclick={() => attempt(() => api.openProfileFolder(profile.id, folder))}>{folder}</button>
      {/each}
    </div>
  </section>

  <section class="card line">
    <div>
      <h3 class="danger">{t("psettings.delete")}</h3>
      <p class="hint">{t("psettings.deleteHint")}</p>
    </div>
    <button class="btn danger" onclick={remove}>
      <Icon name="trash" size={15} />{confirmDelete ? t("psettings.confirmDelete") : t("psettings.delete")}
    </button>
  </section>
</div>

<style>
  .stack {
    max-width: 720px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .card {
    padding: 16px;
    border-radius: 8px;
    background: var(--panel);
    border: 2px solid #1d1a38;
  }

  h3 {
    margin-bottom: 12px;
    font-family: var(--pixel);
    font-size: 15px;
  }

  h3.danger {
    margin-bottom: 2px;
    color: var(--danger);
  }

  .hint {
    font-size: 12px;
    color: var(--text-3);
  }

  .save {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 14px;
  }

  .warn {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--warning);
  }

  .line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
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
    width: 16px;
    height: 16px;
    accent-color: var(--green);
  }

  .folders {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .folders .btn {
    height: 32px;
    font-size: 12.5px;
  }
</style>
