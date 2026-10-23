<script lang="ts">
  import { t } from "../i18n.svelte";
  import { toIconDataUrl } from "../images";
  import { pixelIconKeys } from "../pixelIcons";
  import { toast } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  // `extra` is one more picture to offer, such as a modpack's own icon.
  let {
    value,
    onchange,
    size = 40,
    extra = null,
  }: { value: string; onchange: (icon: string) => void; size?: number; extra?: string | null } = $props();

  let input = $state<HTMLInputElement>();
  const uploaded = $derived(value.startsWith("data:image/") && value !== extra);

  async function upload(file: File | undefined) {
    if (!file) return;
    try {
      onchange(await toIconDataUrl(file));
    } catch {
      toast(t("form.uploadFailed"), "error");
    }
    if (input) input.value = "";
  }
</script>

<div class="grid" role="radiogroup" aria-label={t("form.icon")} style:--tile="{size + 6}px">
  {#if extra}
    <button type="button" role="radio" aria-checked={value === extra} aria-label={t("form.packIcon")} class:on={value === extra} onclick={() => onchange(extra!)}>
      <ProfileIcon icon={extra} {size} />
    </button>
  {/if}
  {#each pixelIconKeys as key (key)}
    <button type="button" role="radio" aria-checked={value === key} aria-label={key} class:on={value === key} onclick={() => onchange(key)}>
      <ProfileIcon icon={key} {size} />
    </button>
  {/each}
  {#if uploaded}
    <button type="button" role="radio" aria-checked="true" aria-label={t("form.upload")} class="on">
      <ProfileIcon icon={value} {size} />
    </button>
  {/if}
  <button type="button" class="upload" title={t("form.upload")} aria-label={t("form.upload")} onclick={() => input?.click()}>
    <Icon name="upload" size={17} />
  </button>
  <input bind:this={input} type="file" accept="image/png,image/jpeg,image/webp,image/gif" hidden onchange={(e) => upload(e.currentTarget.files?.[0])} />
</div>

<style>
  .grid {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  button {
    width: var(--tile);
    height: var(--tile);
    display: grid;
    place-items: center;
    border-radius: 6px;
    border: 2px solid transparent;
  }

  button:hover {
    border-color: var(--border-strong);
  }

  button.on {
    border-color: var(--green-text);
  }

  .upload {
    border: 2px dashed var(--border-strong);
    color: var(--text-2);
  }

  .upload:hover {
    color: var(--text);
    border-color: var(--text-2);
  }
</style>
