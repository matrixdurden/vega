<script lang="ts">
  import { pixelIconUrl } from "../pixelIcons";

  // `icon` is a built-in pixel icon key or an uploaded data: URL.
  let { icon, size = 40 }: { icon: string; size?: number } = $props();

  const uploaded = $derived(icon.startsWith("data:image/"));
  const src = $derived(uploaded ? icon : pixelIconUrl(icon));
  // Whole multiples of 16 keep every pixel of the art the same size.
  const art = $derived(Math.max(16, Math.floor((size * 0.8) / 16) * 16));
</script>

<span class="slot" style:width="{size}px" style:height="{size}px">
  {#if uploaded}
    <img class="photo" {src} alt="" />
  {:else}
    <img class="pixel" {src} alt="" width={art} height={art} />
  {/if}
</span>

<style>
  .slot {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    overflow: hidden;
    border-radius: 4px;
    background: #2b2b32;
    border: 1px solid #3a3a44;
  }

  .pixel {
    image-rendering: pixelated;
  }

  .photo {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
