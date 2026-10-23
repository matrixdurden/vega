<script lang="ts">
  import { t } from "../i18n.svelte";
  import { pixelIconKeys } from "../pixelIcons";
  import { toast } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  let { value, onchange }: { value: string; onchange: (icon: string) => void } = $props();

  const SIZE = 96;
  let input = $state<HTMLInputElement>();
  const uploaded = $derived(value.startsWith("data:image/"));

  // Squares and shrinks the picture so profiles.json stays small.
  async function upload(file: File | undefined) {
    if (!file) return;
    try {
      const bitmap = await createImageBitmap(file);
      const side = Math.min(bitmap.width, bitmap.height);
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = SIZE;
      const ctx = canvas.getContext("2d")!;
      // Keep pixel art crisp when it is scaled up.
      ctx.imageSmoothingEnabled = side > SIZE;
      ctx.drawImage(bitmap, (bitmap.width - side) / 2, (bitmap.height - side) / 2, side, side, 0, 0, SIZE, SIZE);
      onchange(canvas.toDataURL("image/png"));
    } catch {
      toast(t("form.uploadFailed"), "error");
    }
    if (input) input.value = "";
  }
</script>

<div class="grid" role="radiogroup" aria-label={t("form.icon")}>
  {#each pixelIconKeys as key (key)}
    <button type="button" role="radio" aria-checked={value === key} aria-label={key} class:on={value === key} onclick={() => onchange(key)}>
      <ProfileIcon icon={key} size={40} />
    </button>
  {/each}
  {#if uploaded}
    <button type="button" role="radio" aria-checked="true" aria-label={t("form.upload")} class="on">
      <ProfileIcon icon={value} size={40} />
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
    width: 46px;
    height: 46px;
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
