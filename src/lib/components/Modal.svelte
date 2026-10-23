<script lang="ts">
  import type { Snippet } from "svelte";
  import { t } from "../i18n.svelte";
  import Icon from "./Icon.svelte";

  // `bare` leaves the whole box to the content: no padding and no title row.
  let {
    title,
    width = 520,
    bare = false,
    onclose,
    children,
  }: { title: string; width?: number; bare?: boolean; onclose: () => void; children: Snippet } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="modal" class:bare role="dialog" aria-modal="true" aria-label={title} style:width="min({width}px, calc(100vw - 32px))">
    {#if !bare}
      <div class="head">
        <h2>{title}</h2>
        <button class="close" aria-label={t("window.close")} onclick={onclose}><Icon name="x" /></button>
      </div>
    {/if}
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.6);
    animation: fade 0.15s ease;
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  .modal {
    max-height: calc(100vh - 48px);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 20px;
    border-radius: 6px;
    background: var(--panel);
    border: 1px solid var(--border-strong);
    box-shadow: 0 24px 60px rgb(0 0 0 / 0.5);
    animation: rise 0.18s var(--ease);
  }

  .modal.bare {
    padding: 0;
    gap: 0;
    overflow: hidden;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h2 {
    font-size: 18px;
  }

  .close {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-2);
  }

  .close:hover {
    background: var(--panel-2);
    color: var(--text);
  }
</style>
