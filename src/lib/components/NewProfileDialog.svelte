<script lang="ts">
  import type { Loader } from "../api";
  import { t } from "../i18n.svelte";
  import { app, createProfile, errorText, loadVersions, toast } from "../state.svelte";
  import Icon from "./Icon.svelte";

  let loader = $state<Loader>("fabric");
  let version = $state("");
  let name = $state("");
  let creating = $state(false);

  const choices = $derived(app.versions.filter((v) => loader === "vanilla" || v.fabric));
  const placeholder = $derived(`${loader === "fabric" ? "Fabric" : "Vanilla"} ${version}`);

  $effect(() => {
    loadVersions();
  });

  // Keep a valid version selected when the list or loader changes.
  $effect(() => {
    if (!choices.some((v) => v.id === version)) version = choices[0]?.id ?? "";
  });

  function close() {
    app.newProfileOpen = false;
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!version || creating) return;
    creating = true;
    try {
      await createProfile(name.trim() || placeholder, version, loader);
      close();
    } catch (err) {
      toast(errorText(err), "error");
    } finally {
      creating = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} />

<div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && close()}>
  <form class="dialog" onsubmit={submit} aria-labelledby="new-profile-title">
    <div class="head">
      <h2 id="new-profile-title">{t("profile.new")}</h2>
      <button type="button" class="icon-btn" aria-label={t("profile.cancel")} onclick={close}><Icon name="x" /></button>
    </div>

    <div class="field">
      <span class="label">{t("profile.loader")}</span>
      <div class="segmented">
        <button type="button" class:on={loader === "fabric"} onclick={() => (loader = "fabric")}>Fabric</button>
        <button type="button" class:on={loader === "vanilla"} onclick={() => (loader = "vanilla")}>Vanilla</button>
      </div>
      <span class="hint">{t("profile.fabricHint")}</span>
    </div>

    <label class="field">
      <span class="label">{t("profile.version")}</span>
      <select class="input" bind:value={version} disabled={!choices.length}>
        {#each choices as choice (choice.id)}
          <option value={choice.id}>{choice.id}</option>
        {/each}
      </select>
    </label>

    <label class="field">
      <span class="label">{t("profile.name")}</span>
      <input class="input" bind:value={name} {placeholder} maxlength="40" />
    </label>

    <div class="actions">
      <button type="button" class="btn" onclick={close}>{t("profile.cancel")}</button>
      <button type="submit" class="btn primary" disabled={!version || creating}>
        {#if creating}<Icon name="loader" size={16} class="spin" />{/if}
        {t("profile.create")}
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
    width: min(420px, calc(100vw - 32px));
    display: flex;
    flex-direction: column;
    gap: 18px;
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

  .field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .label {
    font-size: 13px;
    color: var(--text-2);
  }

  .hint {
    font-size: 12px;
    color: var(--text-3);
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
    height: 32px;
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
    margin-top: 4px;
  }
</style>
