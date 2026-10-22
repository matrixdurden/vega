<script lang="ts">
  let { url, title, size = 40 }: { url: string | null; title: string; size?: number } = $props();
</script>

{#if url}
  <img class="mod-icon" src={url} alt="" loading="lazy" style:width="{size}px" style:height="{size}px" />
{:else}
  <span class="mod-icon placeholder" style:width="{size}px" style:height="{size}px">{title.slice(0, 1)}</span>
{/if}

<style>
  .mod-icon {
    flex-shrink: 0;
    border-radius: 10px;
    object-fit: cover;
    background: var(--surface);
  }

  .placeholder {
    display: grid;
    place-items: center;
    font-weight: 600;
    color: #cecbf6;
    background: var(--accent-soft);
  }
</style>
