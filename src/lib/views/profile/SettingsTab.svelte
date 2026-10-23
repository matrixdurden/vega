<script lang="ts">
  import { untrack } from "svelte";
  import { api, toDraft, type Profile, type ProfileDraft } from "../../api";
  import Icon from "../../components/Icon.svelte";
  import IconPicker from "../../components/IconPicker.svelte";
  import MemorySlider from "../../components/MemorySlider.svelte";
  import { t } from "../../i18n.svelte";
  import { app, attempt, deleteProfile, loadVersions, updateProfile } from "../../state.svelte";

  let { profile }: { profile: Profile } = $props();

  // Every change saves itself; the form keeps its own copy while typing.
  let draft = $state<ProfileDraft>(untrack(() => toDraft(profile)));
  let confirmDelete = $state(false);

  const choices = $derived(app.versions.filter((v) => draft.loader === "vanilla" || v.fabric));
  const globalMb = $derived(app.settings?.memoryMb ?? 4096);

  $effect(() => {
    loadVersions();
  });

  let timer: ReturnType<typeof setTimeout>;
  function save(patch: Partial<ProfileDraft>, delay = 0) {
    Object.assign(draft, patch);
    clearTimeout(timer);
    const snapshot = $state.snapshot(draft);
    timer = setTimeout(() => attempt(() => updateProfile(profile.id, snapshot)), delay);
  }

  function setLoader(loader: ProfileDraft["loader"]) {
    // Vanilla lists every release; Fabric only the ones it supports.
    const version = loader === "fabric" && !app.versions.find((v) => v.id === draft.gameVersion)?.fabric
      ? (app.versions.find((v) => v.fabric)?.id ?? draft.gameVersion)
      : draft.gameVersion;
    save({ loader, gameVersion: version });
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

<div class="settings">
  <div class="row">
    <span class="label">{t("form.name")}</span>
    <input class="input" value={draft.name} maxlength="40" oninput={(e) => save({ name: e.currentTarget.value }, 500)} />
  </div>

  <div class="row top">
    <span class="label">{t("form.icon")}</span>
    <IconPicker value={draft.icon} onchange={(icon) => save({ icon })} />
  </div>

  <div class="row">
    <span class="label">{t("form.version")}</span>
    <div class="pair">
      <div class="segmented">
        <button class:on={draft.loader === "fabric"} onclick={() => setLoader("fabric")}>Fabric</button>
        <button class:on={draft.loader === "vanilla"} onclick={() => setLoader("vanilla")}>Vanilla</button>
      </div>
      <select class="input" value={draft.gameVersion} onchange={(e) => save({ gameVersion: e.currentTarget.value })}>
        {#if !choices.some((v) => v.id === draft.gameVersion)}<option value={draft.gameVersion}>{draft.gameVersion}</option>{/if}
        {#each choices as choice (choice.id)}<option value={choice.id}>{choice.id}</option>{/each}
      </select>
    </div>
  </div>

  <div class="row">
    <span class="label">{t("psettings.memory")}</span>
    <div class="pair">
      <label class="check">
        <input type="checkbox" checked={draft.memoryMb === null} onchange={(e) => save({ memoryMb: e.currentTarget.checked ? null : globalMb })} />
        {t("psettings.memoryDefault", { gb: globalMb / 1024 })}
      </label>
      {#if draft.memoryMb !== null}<MemorySlider value={draft.memoryMb} onchange={(memoryMb) => save({ memoryMb }, 400)} />{/if}
    </div>
  </div>

  <div class="row actions">
    <button class="btn" onclick={() => attempt(() => api.openProfileFolder(profile.id))}><Icon name="folder" size={15} />{t("psettings.openFolder")}</button>
    <button class="btn danger" onclick={remove}><Icon name="trash" size={15} />{confirmDelete ? t("psettings.confirmDelete") : t("psettings.delete")}</button>
  </div>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    border-radius: 6px;
    background: var(--panel);
    border: 1px solid var(--border);
  }

  .row {
    display: grid;
    grid-template-columns: 140px minmax(0, 1fr);
    align-items: center;
    gap: 16px;
    padding: 14px 18px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .row.top {
    align-items: start;
  }

  .row.top .label {
    padding-top: 12px;
  }

  .label {
    font-family: var(--mc);
    font-weight: 400;
    font-size: 13px;
    color: var(--text-2);
  }

  .pair {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }

  .pair .segmented {
    width: 220px;
  }

  .pair select {
    width: 160px;
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

  .actions {
    display: flex;
    justify-content: space-between;
  }
</style>
