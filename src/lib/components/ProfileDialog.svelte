<script lang="ts">
  import { untrack } from "svelte";
  import { toDraft, type Loader } from "../api";
  import { t } from "../i18n.svelte";
  import { profileIconKeys } from "../profileIcons";
  import { app, createProfile, errorText, loadVersions, toast, updateProfile } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  let { mode, id }: { mode: "create" | "edit"; id?: string } = $props();

  // Snapshot the profile being edited; the form works on its own copy, and
  // App re-creates this dialog whenever `mode` or `id` change.
  const editing = untrack(() => (mode === "edit" ? app.profiles.find((p) => p.id === id) : undefined));
  const initial = editing
    ? toDraft(editing)
    : { name: "", icon: profileIconKeys[(app.profiles.length + 1) % profileIconKeys.length], gameVersion: "", loader: "fabric" as Loader, memoryMb: null };

  let icon = $state(initial.icon);
  let name = $state(initial.name);
  let loader = $state<Loader>(initial.loader);
  let version = $state(initial.gameVersion);
  let saving = $state(false);

  const choices = $derived(app.versions.filter((v) => loader === "vanilla" || v.fabric));
  const placeholder = $derived(`${loader === "fabric" ? "Fabric" : "Vanilla"} ${version}`);
  const versionChanged = $derived(!!editing && (version !== editing.gameVersion || loader !== editing.loader));

  $effect(() => {
    loadVersions();
  });

  // Keep a valid version selected when the list or loader changes.
  $effect(() => {
    if (choices.length && !choices.some((v) => v.id === version)) version = choices[0].id;
  });

  function close() {
    app.dialog = null;
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!version || saving) return;
    saving = true;
    const draft = { name: name.trim() || placeholder, icon, gameVersion: version, loader, memoryMb: initial.memoryMb };
    try {
      if (editing) await updateProfile(editing.id, draft);
      else await createProfile(draft);
      close();
    } catch (err) {
      toast(errorText(err), "error");
    } finally {
      saving = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} />

<div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && close()}>
  <form class="dialog" onsubmit={submit} aria-labelledby="profile-dialog-title">
    <div class="head">
      <h2 id="profile-dialog-title">{editing ? t("dialog.edit") : t("dialog.create")}</h2>
      <button type="button" class="icon-btn" aria-label={t("dialog.cancel")} onclick={close}><Icon name="x" /></button>
    </div>

    <div class="grid">
      <div class="field">
        <span class="label">{t("dialog.icon")}</span>
        <div class="icons" role="radiogroup" aria-label={t("dialog.icon")}>
          {#each profileIconKeys as key (key)}
            <button type="button" role="radio" aria-checked={icon === key} aria-label={key} class="pick" class:on={icon === key} onclick={() => (icon = key)}>
              <ProfileIcon icon={key} size={40} />
            </button>
          {/each}
        </div>
      </div>

      <div class="fields">
        <label class="field">
          <span class="label">{t("dialog.name")}</span>
          <!-- svelte-ignore a11y_autofocus -->
          <input class="input" bind:value={name} {placeholder} maxlength="40" autofocus />
        </label>

        <div class="field">
          <span class="label">{t("dialog.loader")}</span>
          <div class="segmented">
            <button type="button" class:on={loader === "fabric"} onclick={() => (loader = "fabric")}>Fabric</button>
            <button type="button" class:on={loader === "vanilla"} onclick={() => (loader = "vanilla")}>Vanilla</button>
          </div>
          <span class="hint">{t("dialog.loaderHint")}</span>
        </div>

        <label class="field">
          <span class="label">{t("dialog.version")}</span>
          <select class="input" bind:value={version} disabled={!choices.length}>
            {#each choices as choice (choice.id)}
              <option value={choice.id}>{choice.id}</option>
            {/each}
          </select>
          {#if versionChanged && app.mods.length}
            <span class="hint warn"><Icon name="alert" size={14} />{t("dialog.modsWarning")}</span>
          {/if}
        </label>
      </div>
    </div>

    <div class="actions">
      <button type="button" class="btn" onclick={close}>{t("dialog.cancel")}</button>
      <button type="submit" class="btn primary" disabled={!version || saving}>
        {#if saving}<Icon name="loader" size={16} class="spin" />{/if}
        {editing ? t("dialog.save") : t("dialog.createButton")}
      </button>
    </div>
  </form>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: rgb(5 4 15 / 0.6);
    backdrop-filter: blur(6px);
    animation: fade 0.18s ease;
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  .dialog {
    width: min(600px, calc(100vw - 32px));
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 22px;
    border-radius: 18px;
    background: var(--bg-raised);
    border: 1px solid var(--border-strong);
    box-shadow: 0 30px 80px rgb(0 0 0 / 0.5);
    animation: rise 0.22s var(--ease);
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.98);
    }
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h2 {
    font-size: 18px;
    font-weight: 600;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-3);
  }

  .icon-btn:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .grid {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 22px;
  }

  .icons {
    display: grid;
    grid-template-columns: repeat(4, 48px);
    gap: 6px;
  }

  .pick {
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
    border-radius: 14px;
    border: 2px solid transparent;
    transition: border-color 0.15s, transform 0.15s var(--ease);
  }

  .pick:hover {
    transform: scale(1.05);
  }

  .pick.on {
    border-color: var(--text);
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }

  .label {
    font-size: 13px;
    color: var(--text-2);
  }

  .hint {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-3);
  }

  .hint.warn {
    color: #fac775;
  }

  .segmented {
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: 4px;
    gap: 4px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .segmented button {
    height: 30px;
    border-radius: 7px;
    color: var(--text-2);
    font-weight: 500;
  }

  .segmented button.on {
    background: var(--accent);
    color: #fff;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
