<script lang="ts">
  import { pixelRuns } from "../pixelIcons";

  // `icon` is a built-in pixel icon key or an uploaded data: URL.
  let { icon, size = 40 }: { icon: string; size?: number } = $props();

  const uploaded = $derived(icon.startsWith("data:image/"));
  const runs = $derived(uploaded ? [] : pixelRuns(icon));
</script>

<span class="slot" style:width="{size}px" style:height="{size}px">
  {#if uploaded}
    <img src={icon} alt="" />
  {:else}
    <svg viewBox="0 0 16 16" width={Math.round(size * 0.72)} height={Math.round(size * 0.72)} shape-rendering="crispEdges" aria-hidden="true">
      {#each runs as run, i (i)}
        <rect x={run.x} y={run.y} width={run.w} height="1" fill={run.fill} />
      {/each}
    </svg>
  {/if}
</span>

<style>
  /* An inventory slot: dark, square, with a bevelled edge. */
  .slot {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    overflow: hidden;
    border-radius: 4px;
    background: #1d1b31;
    box-shadow:
      inset 2px 2px 0 rgb(0 0 0 / 0.35),
      inset -2px -2px 0 rgb(255 255 255 / 0.06);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
