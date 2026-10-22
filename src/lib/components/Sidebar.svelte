<script lang="ts">
  import { t } from "../i18n.svelte";
  import { app, selectProfile } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";
</script>

<nav>
  <div class="brand" data-tauri-drag-region>
    <span class="logo">
      <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
        <path d="M12 2l2.2 7.8L22 12l-7.8 2.2L12 22l-2.2-7.8L2 12l7.8-2.2z" fill="currentColor" />
      </svg>
    </span>
    <span class="name" data-tauri-drag-region>Vega</span>
  </div>

  <div class="heading">
    <span>{t("nav.profiles")}</span>
    <button class="add" aria-label={t("nav.newProfile")} title={t("nav.newProfile")} onclick={() => (app.dialog = { mode: "create" })}>
      <Icon name="plus" size={16} />
    </button>
  </div>

  <div class="list">
    {#each app.profiles as profile (profile.id)}
      {@const active = app.view === "profile" && profile.id === app.selectedId}
      <button class="profile" class:active aria-current={active ? "page" : undefined} onclick={() => selectProfile(profile.id)}>
        <ProfileIcon icon={profile.icon} size={36} />
        <span class="text">
          <span class="title">{profile.name}</span>
          <span class="meta">{profile.loader === "fabric" ? "Fabric" : "Vanilla"} · {profile.gameVersion}</span>
        </span>
        {#if app.launch.phase !== "idle" && profile.id === app.selectedId}
          <span class="live" class:running={app.launch.phase === "running"}></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="bottom">
    <button class="row" class:active={app.view === "settings"} onclick={() => (app.view = "settings")}>
      <span class="glyph"><Icon name="settings" size={18} /></span>
      {t("nav.settings")}
    </button>
    <button class="row account" onclick={() => (app.view = "settings")}>
      <span class="avatar">{(app.settings?.username ?? "?").slice(0, 1).toUpperCase()}</span>
      <span class="text">
        <span class="title">{app.settings?.username ?? ""}</span>
        <span class="meta">offline</span>
      </span>
    </button>
  </div>
</nav>

<style>
  nav {
    position: relative;
    z-index: 25;
    width: 236px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    padding: 12px 10px;
    background: rgb(18 16 42 / 0.78);
    border-right: 1px solid var(--border);
    backdrop-filter: blur(14px);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 36px;
    padding: 0 8px;
    margin-bottom: 14px;
  }

  .logo {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    background: var(--text);
    color: #26215c;
  }

  .name {
    font-size: 16px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }

  .heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 6px 6px 10px;
    font-size: 12px;
    color: var(--text-3);
  }

  .add {
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-2);
  }

  .add:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: 0 -4px;
    padding: 0 4px;
  }

  .profile,
  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 7px 8px;
    border-radius: 12px;
    border: 1px solid transparent;
    text-align: left;
    transition: background 0.15s, border-color 0.15s;
  }

  .profile:hover,
  .row:hover {
    background: var(--surface-hover);
  }

  .profile.active,
  .row.active {
    background: var(--accent-soft);
    border-color: rgb(127 119 221 / 0.35);
  }

  .text {
    min-width: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  }

  .meta {
    font-size: 11.5px;
    color: var(--text-3);
  }

  .live {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1.4s ease-in-out infinite;
  }

  .live.running {
    background: var(--success);
  }

  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }

  .bottom {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-top: 10px;
    margin-top: 10px;
    border-top: 1px solid var(--border);
  }

  .row {
    color: var(--text-2);
  }

  .glyph {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    background: var(--surface);
  }

  .avatar {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 9px;
    background: #04342c;
    color: var(--success);
    font-weight: 600;
    font-size: 13px;
  }

  .account .title {
    color: var(--text);
  }
</style>
