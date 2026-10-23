<script lang="ts">
  import type { Loader } from "../api";
  import { t } from "../i18n.svelte";
  import { app, loadVersions } from "../state.svelte";
  import IconPicker from "./IconPicker.svelte";

  let {
    icon = $bindable(),
    name = $bindable(),
    loader = $bindable(),
    version = $bindable(),
    placeholder = "",
  }: { icon: string; name: string; loader: Loader; version: string; placeholder?: string } = $props();

  const choices = $derived(app.versions.filter((v) => loader === "vanilla" || v.fabric));

  $effect(() => {
    loadVersions();
  });

  // Keep a valid version selected when the list or loader changes.
  $effect(() => {
    if (choices.length && !choices.some((v) => v.id === version)) version = choices[0].id;
  });
</script>

<div class="fields">
  <div class="field">
    <span class="label">{t("form.icon")}</span>
    <IconPicker bind:value={icon} />
  </div>

  <label class="field">
    <span class="label">{t("form.name")}</span>
    <input class="input" bind:value={name} {placeholder} maxlength="40" />
  </label>

  <div class="row">
    <div class="field">
      <span class="label">{t("form.loader")}</span>
      <div class="segmented">
        <button type="button" class:on={loader === "fabric"} onclick={() => (loader = "fabric")}>Fabric</button>
        <button type="button" class:on={loader === "vanilla"} onclick={() => (loader = "vanilla")}>Vanilla</button>
      </div>
    </div>
    <label class="field">
      <span class="label">{t("form.version")}</span>
      <select class="input" bind:value={version} disabled={!choices.length}>
        {#if !choices.length}<option value={version}>{version}</option>{/if}
        {#each choices as choice (choice.id)}
          <option value={choice.id}>{choice.id}</option>
        {/each}
      </select>
    </label>
  </div>
</div>

<style>
  .fields {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .label {
    font-size: 12px;
    color: var(--text-3);
  }
</style>
