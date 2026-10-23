<script lang="ts">
  import { t } from "../i18n.svelte";
  import { activeAccount, app, selectProfile } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  const account = $derived(activeAccount());
</script>

<nav>
  <div class="brand" data-tauri-drag-region>
    <svg viewBox="0 0 8 8" width="20" height="20" shape-rendering="crispEdges" aria-hidden="true">
      <path d="M3 0h2v3h3v2H5v3H3V5H0V3h3z" fill="#f5f5f7" />
      <rect x="3" y="3" width="2" height="2" fill="#86e073" />
    </svg>
    <span class="mc" data-tauri-drag-region>Vega</span>
  </div>

  <div class="heading">
    <span class="mc">{t("nav.profiles")}</span>
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
        {#if app.runs[profile.id]}
          <span class="live" class:running={app.runs[profile.id].phase === "running"}></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="bottom">
    <button class="account" onclick={() => (app.accountsOpen = true)}>
      <span class="avatar" class:ms={account?.kind === "microsoft"}>{account ? account.username.slice(0, 1).toUpperCase() : "+"}</span>
      <span class="text">
        <span class="name">{account?.username ?? t("nav.noAccount")}</span>
        {#if account}<span class="meta">{account.kind === "microsoft" ? "Microsoft" : "Offline"}</span>{/if}
      </span>
    </button>
    <button class="gear" class:active={app.view === "settings"} aria-label={t("nav.settings")} title={t("nav.settings")} onclick={() => (app.view = "settings")}>
      <Icon name="settings" size={18} />
    </button>
  </div>
</nav>

<style>
  nav {
    position: relative;
    z-index: 25;
    width: 232px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    padding: 12px 10px;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 8px;
    margin-bottom: 12px;
    font-size: 18px;
  }

  .heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px 6px 8px;
    font-size: 12px;
    color: var(--text-3);
  }

  .heading .mc {
    font-weight: 400;
  }

  .add {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-2);
    border: 1px solid var(--border);
  }

  .add:hover {
    background: var(--panel-2);
    color: var(--text);
  }

  .list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item,
  .account {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px;
    border-radius: 4px;
    text-align: left;
  }

  .item:hover,
  .account:hover,
  .gear:hover {
    background: var(--panel-2);
  }

  .item.active {
    background: var(--select);
    box-shadow: inset 3px 0 0 var(--green-text);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--mc);
    font-size: 13px;
    font-variant-ligatures: none;
    color: var(--text);
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
  }

  .live {
    width: 8px;
    height: 8px;
    background: var(--warning);
  }

  .live.running {
    background: var(--green-text);
  }

  .bottom {
    display: flex;
    gap: 4px;
    padding-top: 10px;
    margin-top: 10px;
    border-top: 1px solid var(--border);
  }

  .avatar {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    border-radius: 4px;
    background: #5a3b1f;
    color: #ffd59a;
    font-family: var(--mc);
    font-weight: 700;
  }

  .avatar.ms {
    background: #1d4d23;
    color: var(--green-text);
  }

  .gear {
    width: 42px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-2);
  }

  .gear.active {
    background: var(--select);
    color: var(--text);
  }
</style>
