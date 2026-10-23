<script lang="ts">
  import { t } from "../i18n.svelte";
  import { activeAccount, app, selectProfile } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  const account = $derived(activeAccount());
</script>

<nav>
  <div class="brand" data-tauri-drag-region>
    <svg viewBox="0 0 8 8" width="22" height="22" shape-rendering="crispEdges" aria-hidden="true">
      <path d="M3 0h2v3h3v2H5v3H3V5H0V3h3z" fill="#f0eefc" />
      <rect x="3" y="3" width="2" height="2" fill="#8a80f0" />
    </svg>
    <span class="pixel" data-tauri-drag-region>Vega</span>
  </div>

  <div class="heading">
    <span>{t("nav.profiles")}</span>
    <button class="add" aria-label={t("nav.newProfile")} title={t("nav.newProfile")} onclick={() => (app.createOpen = true)}>
      <Icon name="plus" size={16} />
    </button>
  </div>

  <div class="list">
    {#each app.profiles as profile (profile.id)}
      {@const active = app.view === "profile" && profile.id === app.selectedId}
      <button class="item" class:active aria-current={active ? "page" : undefined} onclick={() => selectProfile(profile.id)}>
        <ProfileIcon icon={profile.icon} size={34} />
        <span class="name">{profile.name}</span>
        {#if app.launch.phase !== "idle" && profile.id === app.selectedId}
          <span class="live" class:running={app.launch.phase === "running"}></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="bottom">
    <button class="account" onclick={() => (app.accountsOpen = true)}>
      <span class="avatar" class:ms={account?.kind === "microsoft"}>{account ? account.username.slice(0, 1).toUpperCase() : "?"}</span>
      <span class="name">{account?.username ?? t("nav.noAccount")}</span>
    </button>
    <button
      class="gear"
      class:active={app.view === "settings"}
      aria-label={t("nav.settings")}
      title={t("nav.settings")}
      onclick={() => (app.view = "settings")}
    >
      <Icon name="settings" size={18} />
    </button>
  </div>
</nav>

<style>
  nav {
    position: relative;
    z-index: 25;
    width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    padding: 12px 10px;
    background: var(--panel);
    border-right: 2px solid #1d1a38;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 8px;
    margin-bottom: 14px;
    font-size: 20px;
  }

  .heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px 6px 8px;
    font-size: 12px;
    color: var(--text-3);
  }

  .add {
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    border-radius: 4px;
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
  }

  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 6px;
    border-radius: 6px;
    text-align: left;
    color: var(--text-2);
  }

  .item:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .item.active {
    background: var(--accent-soft);
    color: var(--text);
    box-shadow: inset 2px 0 0 var(--accent);
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
  }

  .live {
    width: 8px;
    height: 8px;
    background: var(--accent);
  }

  .live.running {
    background: var(--success);
  }

  .bottom {
    display: flex;
    gap: 4px;
    padding-top: 10px;
    margin-top: 10px;
    border-top: 2px solid #1d1a38;
  }

  .account {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 6px;
    border-radius: 6px;
    text-align: left;
  }

  .account:hover,
  .gear:hover {
    background: var(--surface-hover);
  }

  .avatar {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    border-radius: 4px;
    background: #3a2a1a;
    color: #e8b06a;
    font-family: var(--pixel);
  }

  .avatar.ms {
    background: #0d3a24;
    color: var(--success);
  }

  .gear {
    width: 40px;
    display: grid;
    place-items: center;
    border-radius: 6px;
    color: var(--text-2);
  }

  .gear.active {
    background: var(--accent-soft);
    color: var(--text);
  }
</style>
