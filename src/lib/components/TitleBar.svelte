<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { isTauri } from "../api";
  import { t } from "../i18n.svelte";
  import Icon from "./Icon.svelte";

  const win = isTauri ? getCurrentWindow() : null;
</script>

<header data-tauri-drag-region>
  <div class="controls">
    <button aria-label={t("window.minimize")} onclick={() => win?.minimize()}><Icon name="minus" size={16} /></button>
    <button aria-label={t("window.maximize")} onclick={() => win?.toggleMaximize()}><Icon name="square" size={14} /></button>
    <button class="close" aria-label={t("window.close")} onclick={() => win?.close()}><Icon name="x" size={16} /></button>
  </div>
</header>

<style>
  header {
    position: absolute;
    inset: 0 0 auto 0;
    height: 40px;
    display: flex;
    justify-content: flex-end;
    z-index: 20;
  }

  .controls {
    display: flex;
    padding: 6px;
    gap: 2px;
  }

  button {
    width: 34px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-3);
    transition: background 0.15s, color 0.15s;
  }

  button:hover {
    background: var(--panel-2);
    color: var(--text);
  }

  .close:hover {
    background: #e24b4a;
    color: #fff;
  }
</style>
