<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { isTauri } from "../api";
  import { t, type Key } from "../i18n.svelte";
  import type { IconName } from "../icons";
  import { app, type View } from "../state.svelte";
  import Icon from "./Icon.svelte";

  const items: { view: View; icon: IconName; label: Key }[] = [
    { view: "home", icon: "home", label: "nav.home" },
    { view: "mods", icon: "puzzle", label: "nav.mods" },
    { view: "settings", icon: "sliders", label: "nav.settings" },
  ];

  const repo = "https://github.com/matrixdurden/vega";
  const openRepo = () => (isTauri ? openUrl(repo) : window.open(repo, "_blank"));
</script>

<nav data-tauri-drag-region>
  <div class="logo" title="Vega">
    <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
      <path d="M12 2l2.2 7.8L22 12l-7.8 2.2L12 22l-2.2-7.8L2 12l7.8-2.2z" fill="currentColor" />
    </svg>
  </div>

  {#each items as item (item.view)}
    <button
      class:active={app.view === item.view}
      aria-label={t(item.label)}
      aria-current={app.view === item.view ? "page" : undefined}
      onclick={() => (app.view = item.view)}
    >
      <Icon name={item.icon} size={20} />
      <span class="tip">{t(item.label)}</span>
    </button>
  {/each}

  <div class="spacer" data-tauri-drag-region></div>

  <button aria-label="GitHub" onclick={openRepo}>
    <Icon name="github" size={19} />
    <span class="tip">GitHub</span>
  </button>
</nav>

<style>
  nav {
    position: relative;
    z-index: 10;
    width: 68px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 14px 0;
    background: rgb(18 16 42 / 0.7);
    border-right: 1px solid var(--border);
    backdrop-filter: blur(12px);
  }

  .logo {
    width: 38px;
    height: 38px;
    margin-bottom: 14px;
    display: grid;
    place-items: center;
    border-radius: 12px;
    background: var(--text);
    color: #26215c;
  }

  button {
    position: relative;
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    border-radius: 12px;
    color: var(--text-3);
    transition: background 0.15s, color 0.15s;
  }

  button:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  button.active {
    background: var(--accent);
    color: #fff;
  }

  .tip {
    position: absolute;
    left: calc(100% + 12px);
    padding: 5px 10px;
    border-radius: 8px;
    background: var(--bg-raised);
    border: 1px solid var(--border-strong);
    color: var(--text);
    font-size: 12px;
    white-space: nowrap;
    opacity: 0;
    transform: translateX(-4px);
    pointer-events: none;
    transition: opacity 0.15s, transform 0.15s var(--ease);
  }

  button:hover .tip {
    opacity: 1;
    transform: none;
  }

  .spacer {
    flex: 1;
    align-self: stretch;
  }
</style>
